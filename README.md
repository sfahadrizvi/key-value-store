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

The api will return a json or json array depending upon the request with
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
{
    "key" : "key_name",
    "value" : "key_value",
    "found" : "true or false depending on if the key was found"
}
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
{
    "key" : "key_name",
    "deleted" : "true or false depending on if the key was found"
}
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
