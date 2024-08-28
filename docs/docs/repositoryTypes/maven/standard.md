# Documentation for the Maven Repository Format
This is the layout for Maven Repositories and how they work. Use this document to help you improve nitro_repo or even make your own repository


## Deploying

Maven Deploying is done by sending a PUT request to the location of the file.

[PUT] `{BASE_PATH}/repositories/{storage}/{repository}/{fileName}`

File Name being the path of the location.

The groupId is turned into a path so `com.example` would be `com/example` and then they add the artifactId and version to the end.
So if you have a file that is `com.example:example:1.0.0` the path would be `com/example/example/1.0.0`

### Authentication

Maven uses the basic authentication method. This is done by sending the header `Authorization: Basic base64("{username}:{password}")`


## Downloading

Maven downloading is done by sending a GET request to the location of the file.

[GET] `{BASE_PATH}/repositories/{storage}/{repository}/{fileName}`

### Authentication

If the repository is private then the repository should send a WWW-Authenticate header and will let the client know that it needs to authenticate.

It will use the same basic authentication method as deploying.

