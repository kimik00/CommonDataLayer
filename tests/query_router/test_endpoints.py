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
        # print(entry)
    # print(data)
    insert_test_data(db, data)


def insert_test_metrics(data):
    lines = "\n".join(data)
    # print(lines)
    requests.post("http://localhost:8428/write", lines)


@pytest.fixture(params=['non_existing', 'single_schema', 'multiple_schemas'])
def prepare(request):
    with CdlEnv('.', postgres_config=PostgresConfig()) as env, QueryService('50102', PostgresConfig()) as _:

        data, expected = load_case(request.param, 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:
            db = connect_to_postgres(env.postgres_config)
            insert_test_data(db, data['database_setup'])
            db.close()

            with SchemaRegistry("/tmp/schema",
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


def test_endpoint_multiple(prepare):
    qr, data, sid, expected = prepare

    # Request QR for data
    response = qr.query_get_multiple(sid, data['query_for'])

    json1 = json.dumps(response.json(), sort_keys=True)
    json2 = json.dumps(expected, sort_keys=True)
    assert json1 == json2
    # assert response.json() == expected

    print(data)
    print(expected)


def test_endpoint_single_ds():
    with CdlEnv('.', postgres_config=PostgresConfig()) as env, QueryService('50102', PostgresConfig()) as _:
        data, expected = load_case('query_ds', 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:

            db = connect_to_postgres(env.postgres_config)
            insert_test_data(db, data['database_setup'])
            db.close()

            with SchemaRegistry("/tmp/schema",
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

                # Request QR for data
                response = qr.query_get_single(sid, data['query_for'], "{}")

                json1 = json.dumps(response.json(), sort_keys=True)
                json2 = json.dumps(expected, sort_keys=True)
                assert json1 == json2
                # assert response.json() == expected

                print(data)
                print(expected)


def test_endpoint_single_ts():
    with CdlEnv('.') as env, QueryServiceTs('50104', VictoriaMetricsConfig()) as _:
        data, expected = load_case('query_ts', 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:

            insert_test_metrics(data['database_setup'])

            with SchemaRegistry("/tmp/schema",
                                "master",
                                "localhost:9093",
                                "schema_registry",
                                "cdl.schema_registry.internal",
                                "50101"
                                ) as sr:

                sid = sr.create_schema('test_schema',
                                       'cdl.document.input',
                                       'http://localhost:50104',
                                       '{}',
                                       1)

                # Line protocol requires timestamps in [ns]
                # Victoriametrics stores them internally in [ms]
                # but PromQL queries use "unix timestamps" which are in [s]
                start = 1608216910
                end = 1608216919
                step = 1
                req_body = {"from": str(start), "to": str(
                    end), "step": str(step)}

                # print(req_body)

                # export = requests.get("http://localhost:8428/api/v1/export",
                #                       params={'match': '{__name__!=""}'})
                # print(export.text)

                # q = requests.get("http://localhost:8428/api/v1/query_range",
                #                  params={
                #                      'query': '{object_id="6793227c-1b5a-413c-b310-1a86dc2d3c78"}',
                #                      "start": start,
                #                      "end": end,
                #                      "step": step
                #                  })
                # print(q.text)

                # Request QR for data
                response = qr.query_get_single(
                    sid, data['query_for'], json.dumps(req_body))

                json1 = json.dumps(response.json(), sort_keys=True)
                json2 = json.dumps(expected, sort_keys=True)
                assert json1 == json2
                # assert response.json() == expected

                print(data)
                print(expected)


def test_endpoint_schema_ds():
    with CdlEnv('.', postgres_config=PostgresConfig()) as env, QueryService('50102', PostgresConfig()) as _:
        data, expected = load_case('query_ds_by_schema', 'query_router')

        with QueryRouter('1024', '50103', 'http://localhost:50101') as qr:

            with SchemaRegistry("/tmp/schema",
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

                db = connect_to_postgres(env.postgres_config)
                insert_test_document(db, data, sid)
                db.close()

                # Request QR for data
                response = qr.query_get_schema(sid)

                json1 = json.dumps(response.json(), sort_keys=True)
                json2 = json.dumps(expected, sort_keys=True)
                assert json1 == json2
                # assert response.json() == expected

                print(data)
                print(expected)

# Endpoint needs to be fixed
# def test_endpoint_schema_ts():
#     with CdlEnv('.') as env:
#         data, expected = load_case('query_ts_by_schema', 'query_router')
#         with QueryRouter('1024', '50103', 'http://localhost:50101') as _:

#             sid = sr.create_schema('localhost:50101',
#                                          'test_schema',
#                                          'cdl.document.input',
#                                          'http://localhost:50104',
#                                          '{}',
#                                          1)

#             data = [entry.replace(
#                 "{value_replaced_by_test}", sid) for entry in data]

#             insert_test_metrics(data)

#             time.sleep(5)
#             # Line protocol requires timestamps in [ns]
#             # Victoriametrics stores them internally in [ms]
#             # but PromQL queries use "unix timestamps" which are in [s]
#             start = 1608216910
#             end = 1608216919
#             step = 1
#             req_body = {"from": str(start), "to": str(end), "step": str(step)}

#             # print(req_body)

#             # export = requests.get("http://localhost:8428/api/v1/export",
#             #                       params={'match': '{__name__!=""}'})
#             # print(export.text)

#             q = requests.get("http://localhost:8428/api/v1/query_range",
#                              params={
#                                  'query': '{__name__=~\"sid_.*\"}'.replace("sid", sid),
#                                  "start": start,
#                                  "end": end,
#                                  "step": step
#                              })
#             print(q.links)
#             print(q.text)

#             # Request QR for data
#             response = query_get_schema('http://localhost:50103', sid)

#             json1 = json.dumps(response.json(), sort_keys=True)
#             json2 = json.dumps(expected, sort_keys=True)
#             assert json1 == json2
#             # assert response.json() == expected

#             print(data)
#             print(expected)
