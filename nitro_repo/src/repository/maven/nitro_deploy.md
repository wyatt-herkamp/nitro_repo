# Maven Nitro Repository Deploy Format
To unlock all the features of nitro_repo we have a custom deploy mechanism. This system lets the nitro repo take control of the data being passed. This is an optional feature and is not required.

A repository Maven Repository can be configured to require this feature. You can do this to ensure files are signed and other verification is done before deploying

## Notes
- All requests should contain the header `x-nitro-repo-deploy: maven 1`
- While a Deploy is active. All other push and put requests are denied for the project
## Authentication
Authorization is accept in one of the following methods
- `Authorization: Bearer {AUTH_TOKEN}`
- `Authorization: Basic base64("{username}:{password}")`  Accepting this can be disabled in the Repository Configuration

## Definitations
- `{BASE_PATH}` is {url}/repositories/{storage}/{repository}
## Initiating a deploy

[POST] to `{BASE_PATH}/deploy`

### Request Body
As JSON
```json
{
    "group_id": "groupId",
    "artifact_id": "artifactId",
    "name": "Project Name",
    "version": "version",
    "tags": ["array","of","tags"], // Is optional
    "release_type": "Release", // Can be Beta, Alpha, Snapshot
    "files": [
        // Note: A Pom file is required.
        {
            "file_name": "{File Name}",
            "maven_file_type": "", // Pom, Jar, JavadocJar, SourcesJar. Is Optional if it is not standard.
            "sha1": "{SHA1 Hash}",
            "sha512": "{SHA 512 Hash}",
            "md5": "{MD5 Hash}"
        }
        // .. All the other files
    ]
    // TODO: Signature Support
    // Extra Data matching VersionData type in nr-core. All data is optional
    "extra": {}
}
```

### Response:
#### 200
```json
{
    "deploy_id": "SOME_ID" // Will be a UUID
}
```
Can also return unauthorized, forbidden,


## Passing Files

All files should be done as put requests with the path.

Each fileName should match one of the elements from the files in the initial request

[PUT] `{BASE_PATH}/deploy/{deployId}/{fileName}`

### Responses
Each file can return bad request if it doesn't match the hashes provided earlier. Will return conflict if the file was already uploaded
## Cancelling a Deploy

[DELETE] `{BASE_PATH}/deploy/{deployId}`
The deploy will be cancelled
## Publishing
So currently all the files are stored but not available to the public. They are not passed to the storage and are stored with the nitro repo server

[POST] `{BASE_PATH}/deploy/{deployId}/publish`

This will tell nitro_repo to push all results to the storage.

And will be made publicly available.

### Responses
200
```json
{
    "files": [
        {
            "name": "FileName",
            "path": "{BASE_PATH}/{groupId}/{artifactId}/{version}/{file}" // Group id will be split up how you usually see with maven uploads so dev.kingtux will be dev/kingtux
        }
        // .. All other files
    ]
}
```
## Yanking
The header of `x-nitro-repo-deploy` is not required for this request

[DELETE] `{BASE_PATH}/{groupId}/{artifactId}`
Will delete the entire project

[DELETE] `{BASE_PATH}/{groupId}/{artifactId}/{version}`
Will delete a version

[DELETE] `{BASE_PATH}/{groupId}/{artifactId}/{version}/{BUILD_NUMBER}`
Will delete a specific SNAPSHOT build