# Solar Grabber

Solar Grabber is a web service that provides a REST API layer over various
cloud sites/services/APIs to get statistical data of your solar panels.

The services that are currently supported are
[Hoymiles](https://global.hoymiles.com) and
[My Autarco](https://my.autarco.com).

## Building & running

First, you need to provide settings in the file `Rocket.toml` by setting the
username, password and other cloud service-specific settings.
You can copy and modify `Rocket.toml.example` for this and uncomment the part
relevant for the service you want to use.
For example, to configure Solar Grabber to use the My Autarco service:

```toml
[default]
# ...

# Put your solar cloud service settings below and uncomment them based on the
# service you want to use.
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
port = 2399

# ...
```

This will work independent of the type of build. For more about Rocket's
configuration, see: <https://rocket.rs/v0.5-rc/guide/configuration/>.

### Using cargo

Using Cargo it is easy to build and run Solar Grabber. just run:

```shell
$ cargo run --release
...
   Compiling solar-grabber v0.1.0 (/path/to/solar-grabber)
    Finished release [optimized] target(s) in 9m 26s
     Running `/path/to/solar-grabber/target/release/solar-grabber`
```

### Using Docker (Compose)

Using `docker-compose` it is easy (to build and) run using a Docker image.
If you do not change `docker-compose.yml` it will use `Rocket.toml` from
the current working directory as configuration:

```console
$ docker-compose up
...
```

To use Docker directly, run to build an image and the run it:

```console
$ docker build --rm --tag solar-grabber:latest .
...
$ docker run --rm -v ./Rocket.toml:/app/Rocket.toml -p 2399:8000 solar-grabber-latest
...
```

This also uses `Rocket.toml` from the current working directory as configuration.
You can alternatively pass a set of environment variables instead. See
`docker-compose.yml` for a list.

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
{"current_w":23.0,"total_kwh":6159.0,"last_updated":1661194620}
```

This contains the current production power (`current_w`) in Watt,
the total of produced energy since installation (`total_kwh`) in kilowatt-hour
and the (UNIX) timestamp that indicates when the information was last updated.

## Integration in Home Assistant

To integrate the Solar Grabber service in your [Home Assistant](https://www.home-assistant.io/)
installation, add the following three sensor entity definitions to your
configuration YAML and restart:

```yaml
sensors:
  # ...Already exiting sensor definitions...

  - platform: rest
    name: "Photovoltaic Invertor"
    resource: "http://solar-grabber.domain.tld:2399"
    json_attributes:
      - current_w
      - total_kwh
      - last_updated
    value_template: >
      {% if value_json["current_w"] == 0 %}
        off
      {% elif value_json["current_w"] > 0 %}
        on
      {% endif %}

  - platform: rest
    name: "Photovoltaic Invertor Power Production"
    resource: "http://solar-grabber.domain.tld:2399"
    value_template: '{{ value_json.current_w }}'
    unit_of_measurement: W
    device_class: power

  - platform: rest
    name: "Photovoltaic Invertor Total Energy Production"
    resource: "http://solar-grabber.domain.tld:2399"
    value_template: '{{ value_json.total_kwh }}'
    unit_of_measurement: kWh
    device_class: energy
```

This assumes your Solar Grabber is running at <http://solar-grabber.domain.tld:2399>.
Replace this with the URL where Solar Grabber is actually running.
Also, feel free to change the names of the sensor entities.

These sensors use the RESTful sensor integration, for more information see the
[RESTful sensor documentation](https://www.home-assistant.io/integrations/sensor.rest/).

## License

Solar Grabber is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
