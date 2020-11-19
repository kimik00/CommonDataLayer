# Getting Started

# Installation

CDL is a written in Rust. See Rust's [installation][installation] guide to install. Below are the pre-requesites needed to get started: 

- Rust
- Docker
- Docker Compose

You can download [docker desktop][docker-desktop] for both Windows and MacOS to intall docker and docker compose on your local machine.

## Working with CDL Locally

Below is a following simple amount of steps to getting started working with the services in the CDL locally on your machine. To build and install container images of services within the cd, run `build.sh`
in root directory of this project


Please review how to set up CDL locally on your machine but viewing [local setup][local-setup] documentations for a sample deployment. Code below references those configurations in

Below we will walk through a simple use case of the CDL:

**Use Case**
- Create Schema
- Insert Data
- Query Data

## Add Schema via CLI
A schema can be added through the CLI tool localed in the `cdl-cli` directory. To be able to run
the cli you must have a rust compiler. The following command below creates the schema with a name according a json schema in a file as well as sets the topic for routing data through kafka. 

```
cargo run --bin cdl -- --registry-addr <registry_address> schema add --name <schema_name> --topic "cdl.document.input" --file <file_path_to_json>
```

Here is the typical sample JSON schema format that the CDL anticipates and ultimatley will validate data by. Please review [README][schema-readme] in `schema-registry` directory for more information.

```
{
	"$schema": "http://json-schema.org/draft-07/schema#",
    "$id": "http://example.com/product.schema.json",
	"definitions": {
		"1.0.0": {
            "description": "A work order",
            "type": "object",
            "properties": {
                "property1": {
                    "description": "",
                    "type": "integer"
                },
                "property2": {
                    "description":"",
                    "type": "string" 
                },
            },
            "required": ["property1"]
        }
    }
}
```

NOTE: Schema's can be added via gRPC[grpc] to the schema registry. Ensure that you have `protoc` installed on your machine you machine generate [proto](./schema-registry/proto) files in a supported language and make requests via a client


## Insert Data

Data can be inserted into the system by data being written to Kafka or ingested through RabbitMQ. Data must be in JSON format with the following fields: `schemaId`, `objectId` and `data` to be routed through the CDL

Below is an example of the what input data would look like. Both ID fields are UUIDs
```
{
    "schemaId": <UUID>,
    "objectId": <UUID>,
    "data": "{ \"some_propery": \"object\"}"

}
```

### Publish messae via RabbitMQ
Below is a sample `curl` command you can also publish a message through RabbitMQ web admin tool through a exchange or directly to a queue. Review [local setup][local-setup] for configuration details on Kafka, RabbitMQ.

The command below example takes input data and publishes to the default exchange in RabbitMQ. The
message gets consumed and is sent to kafka and published to topic which is determined by `schemaId`. 
The message is then routed to command service which handles routing and storage of data by type. 

```
curl -i -u ${user}:${pass} -H "Accept: application/json" -H "Content-Type:application/json" -XPOST -d'{"properties":{},"routing_key":"my_key","payload":"my body","payload_encoding":"string"}' http://${ampq_url}/api/exchanges/%2F/${exchange}/publish
```

Here is another example:
```
curl -i -u guest:guest -H "Accept: application/json" -H "Content-Type:application/json" -XPOST -d'{"properties":{},"routing_key":"cdl.data.input","payload_encoding":"string","payload":"{\"schemaId\":\"6d7138ec-2a5a-11eb-8004-000000000000\",\"objectId\":\"6bbc9e25-d817-4925-8943-e3c0876768ea\",\"data\":\"{}\"}"}' http://localhost:15672/api/exchanges/%2f/cdl/publish
```

### Publish message to Kafka
Kafka is another entry point to the system. Below is a way to publish a message to specific topic.
Once a message is published to the topic, it is routed to the correct storage repo.

```
./start-kafka-shell.sh <host_ip:9092>
```


## Query Data via Query Service
Following this example local deploymet, you can query for data saved. Here data is saved within
the PR 
```
./bin/query_service --postgres-query-url postgresql://postgres:1234@postgres1:5432/postgres --input-port 50052
```
INPUT_PORT=50052
http://localhost:5432 # Access Postgres DB running


## Query Data via GRPC
Must write your own gRPC client
http://localhhost:58102 # Accesing a GRPC server running on `cdl-document-storage` container



# Deployment

See [Kubernetes-Local-Deployment.md][deployment]


[installation]: https://www.rust-lang.org/tools/install
[deployment]: ./docs/K8s-Local-Deployment.md
[docker-desktop]: https://docs.docker.com/desktop/
[schema-readme]: ./schema-registry/README.md
[grpc]: https://grpc.io/docs/what-is-grpc/introduction/