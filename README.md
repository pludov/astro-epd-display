Driver for the EPD display

Render a state into an image and display it on an EPD display.

The state may be obtained by running a command that outputs a JSON object, or by using HTTP PUT method on `/state` endpoint.

For a demo:

```bash
astro-epd-display -scrape-command mobindi/scrape.sh --template mobindi/template.yaml
```
