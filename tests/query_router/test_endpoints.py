import time
import json
import pytest
import requests

from tests.common import load_case
from tests.common.cdl_env import CdlEnv
from tests.common.query_router import QueryRouter
from tests.common.schema_registry import SchemaRegistry
from tests.common.query_service import QueryService
from tests.common.query_service_ts import QueryServiceTs
from tests.common.config import PostgresConfig, VictoriaMetricsConfig
from tests.common.postgres import connect_to_postgres, insert_test_data


def insert_test_document(db, data, sid):
    for entry in data:
        entry['schema_id'] = sid
    insert_test_data(db, data)


def insert_test_metrics(data):
    lines = "\n".join(data)
    requests.post("http://localhost:8428/write", lines)


def assert_json(lhs, rhs):
    assert json.dumps(lhs, sort_keys=True) == json.dumps(rhs, sort_keys=True)


@pytest.fixture(params=['non_existing', 'single_schema', 'multiple_schemas'])
def prepare(request, tmp_path):
    with CdlEnv('.', postgres_config=PostgresConfig()) as env, QueryService('50102', PostgresConfig()) as _:

        data, expected = load_case(request.param, 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:
            db = connect_to_postgres(env.postgres_config)
            insert_test_data(db, data['database_setup'])
            db.close()

            with SchemaRegistry(str(tmp_path),
                                "master",
                                "localhost:9093",
                                "schema_registry",
                                "cdl.schema_registry.internal",
                                "50101"
                                ) as sr:

                sid = sr.create_schema('test_schema',
                                       'cdl.document.input',
                                       'http://localhost:50102',
                                       '{}',
                                       0)

                yield qr, data, sid, expected


@pytest.fixture
def prepare_env(tmp_path):
    with CdlEnv('.', postgres_config=PostgresConfig()) as env:
        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:
            with SchemaRegistry(str(tmp_path),
                                "master",
                                "localhost:9093",
                                "schema_registry",
                                "cdl.schema_registry.internal",
                                "50101"
                                ) as sr:

                yield env, qr, sr


@pytest.fixture
def prepare_document_storage_env(prepare_env):
    env, qr, sr = prepare_env
    with QueryService('50102', PostgresConfig()) as _:
        sid = sr.create_schema('test_schema',
                               'cdl.document.input',
                               'http://localhost:50102',
                               '{}',
                               0)

        yield env, qr, sid


@pytest.fixture
def prepare_timeseries_env(prepare_env):
    env, qr, sr = prepare_env
    with QueryServiceTs('50104', VictoriaMetricsConfig()) as _:
        sid = sr.create_schema('test_schema',
                               'cdl.document.input',
                               'http://localhost:50104',
                               '{}',
                               1)

        yield qr, sid


@pytest.fixture
def prepare_data():
    data, expected = load_case('query_ts', 'query_router')
    insert_test_metrics(data['database_setup'])

    yield data, expected


def test_endpoint_multiple(prepare):
    qr, data, sid, expected = prepare

    response = qr.query_get_multiple(sid, data['query_for'])

    assert_json(response.json(), expected)


def test_endpoint_single_ds(prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case('query_ds', 'query_router')

    db = connect_to_postgres(env.postgres_config)
    insert_test_data(db, data['database_setup'])
    db.close()

    response = qr.query_get_single(sid, data['query_for'], "{}")

    assert_json(response.json(), expected)


def test_endpoint_single_ts(prepare_timeseries_env, prepare_data):
    qr, sid = prepare_timeseries_env
    data, expected = prepare_data

    time.sleep(1)

    # Line protocol requires timestamps in [ns]
    # Victoriametrics stores them internally in [ms]
    # but PromQL queries use "unix timestamps" which are in [s]
    start = 1608216910
    end = 1608216919
    step = 1
    req_body = {"from": str(start), "to": str(
        end), "step": str(step)}

    response = qr.query_get_single(
        sid, data['query_for'], json.dumps(req_body))

    assert_json(response.json(), expected)


def test_endpoint_schema_ds(prepare_document_storage_env):
    env, qr, sid = prepare_document_storage_env
    data, expected = load_case('query_ds_by_schema', 'query_router')

    db = connect_to_postgres(env.postgres_config)
    insert_test_document(db, data, sid)
    db.close()

    response = qr.query_get_schema(sid)

    assert_json(response.json(), expected)
