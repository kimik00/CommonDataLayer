# Data Router

### Responsibilities

The data router is responsible for taking in input data and routing it to the correct storage based on 
the data's schema and its associated topic. 

### Communication

The data router routes requests from RabbitMQ and Kafka to the correct storage solution based on the schema and data type. The data router also communicates with the schema registry based on the `SCHEMA_ID` data is written to a specific topic.

Below is the data needed by data router:

```
{
    "schemaId": <UUID>,
    "objectId": <UUID>,
    "data": "{ \"some_propery": \"object\"}"

}
```

### Configuration

To configure data router, set following environment variables. `INPUT_ADDR` and `INPUT_QUEUE` is related to the incoming data in the router. `KAFKA_BROKERS`, `KAFKA_ERROR_CHANNEL` are related to messages being routed through Kafka to the corresponding command service.


```
INPUT_ADDR
INPUT_QUEUE:
KAFKA_BROKERS:
KAFKA_ERROR_CHANNEL:
SCHEMA_REGISTRY_ADDR
CACHE_CAPACITY
```

See an example [configuration][configuration] of deployment of data router and other services. 



[configuration]: ../examples/deploy/SETUP.md