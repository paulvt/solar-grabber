# Solar Grabber

Solar Grabber is a web service that provides a REST API layer over various
cloud sites/services/APIs to get statistical data of your solar panels.

## Building & running

First, you need to provide settings in the file `Rocket.toml` by setting the
username, password and other cloud service-specific settings.
You can copy and modify `Rocket.toml.example` for this.
For example for My Autarco:

```toml
[default]
# ...

# Put your solar cloud service settings below and uncomment them
[default.service]
kind = "MyAutarco"
username = "foo@domain.tld"
password = "secret"
site_id = "abc123de"
```

You can also change this configuration to use a different address and/or port.
(Note that Rocket listens on `127.0.0.1:8000` by default for debug builds, i.e.
builds when you don't add `--release`.)

```toml
[default]
address = "0.0.0.0"
port = 8080

# ...
```

This will work independent of the type of build. For more about Rocket's
configuration, see: <https://rocket.rs/v0.5-rc/guide/configuration/>.

Finally, using Cargo, it is easy to build and run Solar Grabber, just run:

```shell
$ cargo run --release
...
   Compiling solar-grabber v0.1.0 (/path/to/solar-grabber)
    Finished release [optimized] target(s) in 9m 26s
     Running `/path/to/solar-grabber/target/release/solar-grabber`
```

## API endpoint

The `/` API endpoint provides the current statistical data of your solar panels
once it has successfully logged into the cloud service using your credentials.
There is no path and no query parameters, just:

```http
GET /
```

### Response

A response uses the JSON format and typically looks like this:

```json
{"current_w":23,"total_kwh":6159,"last_updated":1661194620}
```

This contains the current production power (`current_w`) in Watt,
the total of produced energy since installation (`total_kwh`) in kilowatt-hour
and the (UNIX) timestamp that indicates when the information was last updated.

## License

Solar Grabber is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
