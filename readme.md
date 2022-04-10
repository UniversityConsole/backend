# uc-service

This repository holds the source code for all the backend services in the University Console system.

# Testing services

Every service (with the exception of Frontend service) understands and responds to gRPC traffic.

## Remote service

Invoking an actual running service is a bit complicated as services run in private subnets of the VPC. However, the VPC offers some Bastions that can be used to access the private network using the appropriate certificate.

Start by creating an SSH tunnel:

```
ssh -o ServerAliveInterval=60 -f -N -L LocalPort:TaskIp:8080 -i KeyFile ec2-user@BastionIp
```

Substitute:
* `LocalPort` with the port you wish to use locally. The service will be accessible over `localhost:LocalPort`.
* `TaskIp` with the private IP address of the task you wish to invoke.
* `KeyFile` with the appropriate `.pem` certificate to be used for authentication.
* `BastionIp` with the public IP address of the Bastion host.

To test out the connection, run

```
curl localhost:LocalPort
```

The output should be similar to:

```
curl: (1) Received HTTP/0.9 when not allowed
```
