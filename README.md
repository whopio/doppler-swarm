# doppler-swarm

Automate synchronization of Docker Swarm services with Doppler.

## Overview

If you use Docker Swarm, you may have encountered the need to store your configuration somewhere and gracefully restart your services when configuration changes. We use Doppler. Unfortunately, Doppler doesn't offer native support for Docker Swarm out of the box. This tool bridges that gap by providing a seamless integration between Docker Swarm and Doppler.


## Doppler plan requirements

This tool uses a specific Doppler API that enables it to subscribe to configuration changes. Please note that this API is available on Team and Enterprise plans only. For more details, refer to the [Doppler documentation on automatic restart](https://docs.doppler.com/docs/automatic-restart).

## Limitations

1. Please, take into account that this tool rewrites your docker service env vars completely. Do not add any env var to your docker service manually since they will be rewritten.

2. Setup alerts on errors in logs. Configuration is important.

## Getting Started

1. Configure `config.json` with Doppler tokens and Docker service names. Take a look at [example configuration](https://github.com/whopio/doppler-swarm/blob/main/config_example.json). watcher is a single process that subscribes to Doppler and listens for changes in environment.
2. Store `config.json` on one of your docker swarm manager hosts somewhere (for example, at `/etc/doppler-swarm/config.json`).
3. Create a Docker service bound to the manager host (replace `myhostname1` with the actual hostname):
   ```bash
   docker service create \
        --user root \
        --mount type=bind,source=/etc/doppler-swarm/config.json,target=/app/config.json \
        --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
        --name doppler-swarm \
        --constraint "node.role==manager" \
        --constraint "node.hostname==myhostname1" \
        whop/doppler-swarm:latest \
        /app/doppler-swarm /app/config.json
   ```
   Ensure that the service is started by a user with write access to /var/run/docker.sock.
4. Check the logs for any errors: `docker service logs doppler-swarm`

## Have Suggestions or Found Any Errors?

Feel free to [create a new issue](https://github.com/whopio/doppler-swarm/issues) if you have suggestions, found any errors, or need assistance.

## Interested in Improving Configuration Management?

Come [join us](https://careers.whop.com) :)
