# NPM Registry Errors

## Invalid Package Name
NPM Packages must be lowercase, and can only contain letters, numbers, underscores, and dashes. If you try to publish a package with an invalid name, you will receive an error.

### Invalid Tarball URL

If you set your registry url to `{BASE_URL}/repositories/{storage}/{repsotiroy}`. This causes NPM to ignore the last part of the url. You need to add a trailing slash to the end of the url.