import json
import pytest

from tests.common.cdl_env import cdl_env
from tests.common.config import PostgresConfig, VictoriaMetricsConfig
from tests.common.query_router import QueryRouter
from tests.common.query_service import QueryService
from tests.common.query_service_ts import QueryServiceTs
from tests.common.schema_registry import SchemaRegistry


def assert_json(lhs, rhs):
    assert json.dumps(lhs, sort_keys=True) == json.dumps(rhs, sort_keys=True)


@pytest.fixture
def prepare_env(tmp_path):
    with cdl_env('.', postgres_config=PostgresConfig(), victoria_metrics_config=VictoriaMetricsConfig()) as env:
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
        yield env, qr, sid
