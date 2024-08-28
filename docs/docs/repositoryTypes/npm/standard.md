## Publishing

## Logging in

When you run `npm adduser` it will first send a request to `{registry_url}/-/v1/login`
if this path resolves it should return a web login response. However, this feature is not documented here or implemented in Nitro Repo so this returns a bad request.

So if that requests returns a bad request. It will attempt to do a couch db login. 


### Couch DB Login

Which is at route PUT `-/user/org.couchdb.user:{SOME_USER}`

SOME_USER being the username.

The body will be `application/json` with
```json
{
    "name": "Username",
    "password": "Password",
    "email": "Email",
    "login_type": "user",
    "roles": [],
    "date": "CURRENT_ISO_8601_TIMESTAMP"
}
```

You will now validate the login and return a token if successful.

Response is `application/json` with
```json
{
    "token": "TOKEN"
}
```
NPM will save this to the config file and you will be logged in.