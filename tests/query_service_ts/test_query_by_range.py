import pytest
import json
from tests.query_service_ts import prepare_env
from tests.common import load_case
from rpc.proto.query_service_ts_pb2 import Range


@pytest.fixture(params=["range/single"])
def prepare(request, prepare_env):
    db, stub = prepare_env
    data, expected = load_case(request.param, "query_service_ts")
    db.insert_test_data(data['database_setup'])
    query = data["query_for"]
    return stub, expected, query

#TODO: More testcases
def test_query_by_range(prepare):
    stub, expected, query = prepare
    response = stub.QueryByRange(Range(**query))
    assert json.loads(str(response.timeseries)) == expected
