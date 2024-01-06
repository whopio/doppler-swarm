# doppler-swarm

Automate synchronization of Docker Swarm services with Doppler.

## Overview

This program integrates with Docker Swarm and Doppler to update services automatically based on changes in environment variables in Doppler.

## Doppler plan requirements

This tool uses a specific Doppler API that enables it to subscribe to configuration changes. Please note that this API is available on Team and Enterprise plans only. For more details, refer to the [Doppler documentation on automatic restart](https://docs.doppler.com/docs/automatic-restart).

## Getting Started

1. Configure `config.json` with Doppler tokens and Docker service names.
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

## Have a suggestions or found any errors?

Create [a new issue](https://github.com/whopio/doppler-swarm/issues).
