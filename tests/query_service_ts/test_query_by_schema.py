import pytest
import json

from tests.query_service_ts import prepare_env
from tests.common import load_case
from rpc.proto.query_service_ts_pb2 import SchemaId


@pytest.fixture(params=["schema/single"])
def prepare(request, prepare_env):
    db, stub = prepare_env
    data, expected = load_case(request.param, "query_service_ts")
    db.insert_test_data(data['database_setup'])
    query = data["query_for"]
    return db, stub, expected, query

# TODO: Debug why it doesn't work, fix to query by schema instead of name of metrics and more testcases


def test_query_by_schema(prepare):
    db, stub, expected, query = prepare
    print(db.fetch_data_table())
    query_request = SchemaId(**query)
    print(query_request)
    response = stub.QueryBySchema(query_request)
    print(response)
    assert json.loads(str(response.timeseries)) == expected
