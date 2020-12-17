import json
import pytest
import requests
import grpc
import tests.query_router.schema_registry_pb2 as pb2
import tests.query_router.schema_registry_pb2_grpc as pb2_grpc

from tests.common import load_case
from tests.common.cdl_env import CdlEnv
from tests.common.query_router import QueryRouter
from tests.common.config import PostgresConfig
from tests.common.postgres import connect_to_postgres, insert_test_data


def registry_create_schema(url, name, topic, query, body, schema_type):
    with grpc.insecure_channel(url) as channel:
        stub = pb2_grpc.SchemaRegistryStub(channel)
        resp = stub.AddSchema(pb2.NewSchema(
            id="", name=name, topic=topic, query_address=query, definition=body, schema_type=schema_type))
        return resp.id


def query_get_single(url, schema_id, object_id):
    return requests.get(f"{url}/single/{object_id}", headers={'SCHEMA_ID': schema_id})


def query_get_multiple(url, schema_id, object_ids):
    return requests.get(f"{url}/multiple/{object_ids}", headers={'SCHEMA_ID': schema_id})


@pytest.fixture(params=['non_existing', 'single_schema', 'multiple_schemas'])
def prepare(request):
    with CdlEnv('.', postgres_config=PostgresConfig()) as env:
        data, expected = load_case(request.param, 'query_router')

        db = connect_to_postgres(env.postgres_config)

        with QueryRouter('1024', '50103', 'http://localhost:50101') as _:
            insert_test_data(db, data['database_setup'])

            # cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema add
            #   --name ds-schema
            #   --topic cdl.document.input
            #   --query-address "http://postgres_query:50102"
            #   --file schema.json
            #   --schema-type DocumentStorage
            sid = registry_create_schema('localhost:50101',
                                         'test_schema',
                                         'cdl.document.input',
                                         'http://localhost:50102',
                                         '{}',
                                         0)

            # cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema add
            #   --name ds-schema
            #   --topic cdl.document.input
            #   --query-address "http://victoria_query:50104"
            #   --file schema.json
            #   --schema-type Timeseries
            # sid = registry_create_schema('localhost:50101',
            #                        'test_schema',
            #                        'cdl.document.input',
            #                        'http://victoria_query:50104',
            #                        '{}',
            #                        1)

            yield data, sid, expected

        db.close()


def test_endpoint_multiple(prepare):
    data, sid, expected = prepare

    # Request QR for data
    response = query_get_multiple(
        'http://localhost:50103', sid, data['query_for'])

    json1 = json.dumps(response.json(), sort_keys=True)
    json2 = json.dumps(expected, sort_keys=True)
    assert json1 == json2
    # assert response.json() == expected

    print(data)
    print(expected)
