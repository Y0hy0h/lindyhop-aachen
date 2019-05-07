# Development
To run a test server, execute
```bash
yarn fake-backend
```
This will launch a db server and make available the admin at `localhost:3000/admin`.

> Note that you will not be able to load subpages, e. g. `localhost:3000/admin/event/1` directly due to a [bug in the db server](https://github.com/typicode/json-server/issues/591). However, once the admin is loaded from `localhost:3000/admin`, you can click all links and use the browser's backwards and forwards buttons.