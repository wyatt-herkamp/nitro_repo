# Nitro Repository CLI


## Commands

### Login
Adds an instance of a Nitro Repository to your local configuration.
```
nrc login  --url <url> --token <token>
```

### Maven

#### `stage`

### Admin
List Storages
```
nrc admin --storage list
```
Show Storage Details
```
nrc admin --storage <storage>
```
```
nrc admin --storage new --id <name> --type LocalStorage --path <path>
```

Show Repository Details
```
nrc admin --repository <storage>/<repository>
```
