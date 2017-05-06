# rust-swiftclient

Learning rust by implementing a basic swiftclient. Just KeystoneV2
auth and basic operations for the moment.

Uses Reqwest for HTTP client library and Hyper for header handling.

The following environment variables are used by the (very basic)
client and tests:

  * "OS_USERNAME"
  * "OS_PASSWORD"
  * "OS_PROJECT_NAME"
  * "OS_AUTH_URL"
  * "OS_REGION_NAME"

The following environment variables are used only for testing:

  * "TEST_CONTAINER"
  * "TEST_OBJECT"

In the future I will specify what the contents of the test container
and object should be, for now this is unimportant.