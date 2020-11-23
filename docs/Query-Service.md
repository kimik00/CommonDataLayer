# Query Service

### Responsibilites

The query service is reponsible for querying data from several dabtabases.

### Communication

Communication to query service is done through [gRPC][grpc] based on two [endpoints][endpoints] of querying for data by `SCHEMA_ID` or multiple `OBJECT_ID`s. Query service communicates with multiple databases such as postgresql, druid, sled. Query service also communicates with [schema registry][schema-registry]. 


### Configuration

To configure query service, set environmet variables for `INPUT_PORT`, `DS_QUERY_URL` or `POSTGRES_QUERY_URL` to configure the corressponding database.

See an example [configuration][configuration] of deployment of data router and other services. 

[grpc]: https://grpc.io/docs/what-is-grpc/introduction/
[proto]: ../query-service/proto/query.proto
[schema-registry]: ../schema-registry/README.md
[configuration]: ../examples/deploy/SETUP.md
[endpoints]: ../query-service/proto/query.proto