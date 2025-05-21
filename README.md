# KuCo - Kubernetes Console 
KuCo is a Kubernetes Console TUI, used for simplifying repetative commands with a friendly interface.

Inspired by (much better and more interesting) projects such as [Atuin](https://atuin.sh/), and the [Dirtwave M8 Tracker](https://dirtywave.com/), KuCo aims to combine elements 
from each to create something which can stave off my RSI from writing `k get po -n... | grep...` constantly.

KuCo's functionality is encapsulated by a set of screens. The way you traverse those screens is wrapped up in 
a UI element shamelessly stolen from the M8.

TODO: Explain the chain and all that ...

```
  S A 
N P C L
  D D

```

Moving from left to right, prerequisite data is stored for each subsequent screen (ie. which namespace your pod
is in, which pod your containers are in, etc.). Moving up and down on a certain column reveals additional functions 
tied to that specific column.

In KuCo's case, moving up reveals context-specific screens (ie. Pod -> Scale, Container -> Attach). Moving down will 
let you access the same generic screen for the resource type you are currently working with (a generic "Describe" 
screen, exposing information from the K8s API).

The letters match the resource each column is responsible for:
- Column 1: Namespaces
- Column 2: Pods (Scale, Describe)
- Column 3: Containers (Attach, Describe)
- Column 4: Logs

Additionally, KuCo will try to learn from your usage of it via a Sqlite backend. Within a certain Kube Context, metrics
about what your most used resource names (for ephemeral resources, their controller will be queried to get a static name)
will be gathered and used to arrange the lists presented in the main row to automatically select your most used resource 
first, and arrange the runners up close by.

## Local Development Environment
I've included a DevBox (TODO: add links and all that) configuration which will stand up a development environment, complete 
with Rust, Bacon, Kubectl, Docker, and KinD. You can find additional scripts in the `scripts` repository for deploying test 
pods, or running tests of adding and deleting resources.

### Valkey
KuCo uses `redis-rs` to cache Kube data in Valkey. In order to use DevBox's instance of Valkey, please check out the following instructions:

```
valkey NOTES:
Running `devbox services start valkey` will start valkey as a daemon in the background.

You can manually start Valkey in the foreground by running `valkey-server $VALKEY_CONF --port $VALKEY_PORT`.

Logs, pidfile, and data dumps are stored in `.devbox/virtenv/valkey`. You can change this by modifying the `dir` directive in `devbox.d/valkey/valkey.conf`

Services:
* valkey

Use `devbox services start|stop [service]` to interact with services

This plugin creates the following helper files:
* ${PROJECT_PATH}/kuco-rs/devbox.d/valkey/valkey.conf
* ${PROJECT_PATH}/kuco-rs/.devbox/virtenv/valkey/process-compose.yaml

This plugin sets the following environment variables:
* VALKEY_PORT=6379
* VALKEY_CONF=${PROJECT_PATH}/kuco-rs/devbox.d/valkey/valkey.conf

To show this information, run `devbox info valkey`

```

### Project Structure
- kuco
- kuco-backend-k8s
- kuco-backend-sqlite
- kuco-cache

## Technologies Used
- [Sqlite](https://sqlite.org/) 
- Rust ([Ratatui](https://ratatui.rs/), [SQLx](https://github.com/launchbadge/sqlx), [Tokio](https://tokio.rs/), [kube-rs](https://kube.rs/))
- [DevBox](https://www.jetify.com/devbox) 
- [KinD](https://kind.sigs.k8s.io/)

## License

Copyright (c) Alexei Ozerov <aozerov.dev@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE

[![Built with Devbox](https://www.jetify.com/img/devbox/shield_galaxy.svg)](https://www.jetify.com/devbox/docs/contributor-quickstart/)
