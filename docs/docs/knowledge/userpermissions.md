# Available Permissions for Users

1. `Admin` Overrides everything
2. `User Manager`. Can Manage User
3. `Repository Manager`. Creates Repos, Create Storages, and modify settings
4. `Deployer` Can have specified repositories they can access
5. `Viewer`Can have specified repositories they can access. Only matters if repository is set to private. If the deployer permission exists the reader permission is present too. 


## Deployer Special Properties

- `*` is a wild card
- `{}` Will be parsed as json
- `()` are just meant as variables for example
- Available Repository classifiers. `type` Meaning the repository type. Such as `NPM` or `Maven`. `policy` Reference if its `Snapshot`, `Mixed`, `Release`. 

### Examples 

1. `*/*` can deploy to all repositories
2. `(storage_name)/*` Can deploy to all repositories in a specific storage
3. `(storage_name)/(repository)`
4. `(storage_name)/{"type": "npm"}` can deploy to all npm repos
5. `(storage_name)/{"policy": "Snapshot"}` can deploy to all Snapshot. Policy can be combined with type.

Any combination of the rules can be combined so `*/{"policy":"Release", "type": "maven"}` So this will say any storage. But only maven policies set to Release
