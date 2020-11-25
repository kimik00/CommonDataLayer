# Command Service

### Responsibilities

The command service is responsible for consuming a specific input topic and writing that data to its to a variety of different storage solutions. 

### Communication

Communication from the command service is done through through Kafka. An input topic is consumed and the message is routed to the corresponding database based on configuration. 


### Configuration

It is important to ensure that configure command service with the url to a database from eitehr Postgres, Druid or SLED

```
KAFKA_INPUT_GROUP_ID
KAFKA_INPUT_BROKERS
KAFKA_INPUT_TOPIC: 
POSTGRES_OUTPUT_URL
SLEIGH_OUTPUT_ADDR
DRUID_OUTPUT_BROKER
DRUID_OUTPUT_TOPIC
REPORT_BROKER
REPORT_TOPIC
```
