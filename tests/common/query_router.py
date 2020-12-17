import os
import subprocess

from tests.common.config import PostgresConfig

EXE = os.getenv('QUERY_ROUTER_EXE') or 'query-router'


class QueryRouter:
    def __init__(self, cache_capacity, input_port, schema_registry_addr):
        self.cache_capacity = cache_capacity
        self.input_port = input_port
        self.schema_registry_addr = schema_registry_addr

    def __enter__(self):
        env = {}

        env.update(CACHE_CAPACITY=self.cache_capacity)
        env.update(INPUT_PORT=self.input_port)
        env.update(SCHEMA_REGISTRY_ADDR=self.schema_registry_addr)

        self.svc = subprocess.Popen([EXE], env=env)
        return self.svc

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.svc.kill()
