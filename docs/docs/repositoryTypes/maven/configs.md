# Maven Repository


## Maven Push Rules - Options - Hosted Only

- `push_policy`: The Policy for pushing artifacts to the repository. This can be one of the following:
  - `Release`: Only accepts releases. Denies snapshots.
  - `Snapshot`: Only accepts snapshots. Denies releases.
  - `Mixed`: Accepts both releases and snapshots.
- `yanking_allowed`: Whether or not yanking is allowed. This is a boolean value.
- `allow_overwrite`: Whether or not overwriting artifacts is allowed. This is a boolean value.
- `must_be_project_member`: Whether or not the user must be a member of the project to push artifacts. This is a boolean value. When using standard maven deploy. This feature may not work as expected.
- `require_nitro_deploy`: If true standard maven deploy will not work. This is a boolean value.
- `must_use_auth_token_for_push`: If true the user must use an auth token to push artifacts. This is a boolean value. When using standard maven deploy. You can put your auth token in the password field and the username field can be anything.


