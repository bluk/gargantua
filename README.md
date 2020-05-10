# Gargantua

A web application which returns empty responses from the void.

## Configuration

### Environment Variables

#### PORT

The port the application should listen on. Defaults to `8080`.

## Tools

### Docker

#### Build

```
docker build -f deployments/docker/Dockerfile -t gargantua .
```

#### Run

```
docker run -ti -p 8080:8080 --disable-content-trust gargantua
```
