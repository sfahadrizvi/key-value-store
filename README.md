<!-- ABOUT THE PROJECT -->
## This is a simple key valye store

This is simple implementation of a key value store that can be used to store keys in a SSD. 

## Usage
Usage: key_value_store_ssd [OPTIONS]

Options:
  ```
  -p, --port <PORT>  The port to listen on
      --path <PATH>  The path to the key-value store [default: ./ssd]
  -h, --help         Print help
  -V, --version      Print version
  ```

The path needs to exist before the app is started.

## API Endpoints
The store supports the following Apis. All apis require a `POST` request with a `Json body` and need to set `Content-Type: Application/Json` header

### /insert
This api will create a new key or fail if one exists
```
{
    "key":"key_name",
    "value":"key_value"
}
```
or it can be an array of multiple keys values

```
[
    {
        "key" : "key_1_name",
        "value" : "key_1_value"
    },
    {
        "key" : "key_2_name",
        "value" : "key_2_value",
    }
]
```

The api will return a json array depending upon the request with
```
{
    "key" : "key_name",
    "modified" : "true or false depending on the success or failure"
}
```

### /update
This works like the `/insert` endpoint with the addition of creating a file if it does not exist and updating if it exists.

### /get
This api will get the values of keys
```
[
    {
        "key":"key_name"
    }
]
```
or it can be an array of multiple keys values
```
[
    {
        "key" : "key_1_name"
    },
    {
        "key" : "key_2_name"
    }
]
```

The api will return a `Json array` depending upon the request with
```
[
    {
    "key" : "key_name",
    "value" : "key_value",
    "found" : "true or false depending on if the key was found"
    }
]
```
or 
```
[
    {
        "key" : "key_1_name",
        "value" : "key_1_value",
        "found" : "true"
    },
    {
        "key" : "key_2_name",
        "value" : "",
        "found" : "false"
    }
]
```

The value of keys not present in the store will be an empty string. The `found` indicates if the key was found or not in the store.

### /delete
This api will delete keys from the store
```
{
    "key":"key_name"
}
```
or it can be an array of multiple keys values
```
[
    {
        "key" : "key_1_name"
    },
    {
        "key" : "key_2_name"
    }
]
```

The api will return a `Json` or `Json array` depending upon the request with
```
[
    {
        "key" : "key_name",
        "deleted" : "true or false depending on if the key was found"
    }
]
```
or 
```
[
    {
        "key" : "key_1_name",
        "deleted" : "true"
    },
    {
        "key" : "key_2_name",
        "deleted" : "true"
    }
]
```
For safety/security keys `deleted` are always `true` even if the key is not found

### /key
This api will get a list of key mathcing a prefix provided in the body
```
{
    "prefix":"key"
}
```
Unlike the other endpoind this does not take an array but returns an array. The above example will match all keys starting with `key`
```
[
    {
        "key":"key_1_name"
    },
    {
        "key":"key_2_name"
    },
    {
        "key":"key_3_name"
        }
]
```
It will return an empty array is nothing is found

## Sample Test Calls
# insert
```
curl --location 'http://localhost:8080/insert' \
--header 'Content-Type: application/json' \
--data '[
    {
    "key":"key_1_name",
    "value":"key_1_value"
    },
    {
    "key":"key_2_name",
    "value":"key_2_value"
    },
    {
    "key":"key_3_name",
    "value":"key_3_value"
    }
]'
```
On success this will resturn 
```
[
    {"key":"key_1_name", "modified": "true"},
    {"key":"key_2_name", "modified": "true"},
    {"key":"key_3_name", "modified": "true"}
]
```
# get
```
curl --location 'http://localhost:8080/get' \
--header 'Content-Type: application/json' \
--data '
    [
        {"key":"key_1_name"},
        {"key":"key_2_name"},
        {"key":"key_non_existing"}
    ]
'
```
The above example returns 
```
[
    {"key":"key_1_name", "value":"key_1_value" "found": "true"},
    {"key":"key_2_name", "value":"", "found": "false"},
    {"key":"key_non_existing", "value":"", "found": "false"}]
```
The `key_non_existing` has a `found=false` indication that this value was not found.

#keys
```
curl --location 'http://localhost:8080/keys' \
--header 'Content-Type: application/json' \
--data '{
    "prefix": "key"
}'
```
The above example will return keys mathing `key*`
```
[
    {"key":"key_1_name"},
    {"key":"key_3_name"},
    {"key":"key_name"}
]
```

# delete
```
curl --location 'http://localhost:8080/delete' \
--header 'Content-Type: application/json' \
--data '
    [
        {"key":"key_1_name"},
        {"key":"key_2_name"},
        {"key":"key_nonexisting"}
    ]
```
It will delete the 3 files and result something like 
```
    {"key":key_1_name, "deleted": "true"},{"key":key_2_name, "deleted": "true"},{"key":key_nonexisting, "deleted": "true"}
```

## Excluded Tests
The tests are not included because this is writing to files and it can quickly destroy regular ssd. 

## Future Enhancements
- The file system needs to be injected and decoupled from the system to be able to mock it correctly. That will help add tests.
- The code for `get/insert/delete` is similar except the returned `Json` and the actual function of the `FileSystemGateway` called. It can be refactored into same function with different call function as higer order function or simple if logic
- The locking mechanism is simple, it create X amount of locks and the lock bucket is based on the size of the key. It can be imporved by hasing and adding a larger bucket. A hashmap of locks for each key separately might be too memory heavy and will need additional functionality for pruning and separate locks for making the hash map itself thread safe. There are some libraries available for thread safe hashmap.
- The cache is built in the http server. It can be moved to the file system gateway to keep the complexity out of server logic and let the file system gateway handle the cache mechanism.
- The current cache is an external library. A simple hash-map can also be used if it is made thread-safe.




