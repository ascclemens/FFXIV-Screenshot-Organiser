# FFXIV Screenshot Organiser

I opened up my screenshots folder that has been populating over many years, and I realised it's a
mess. Sometimes I want to go back and view older screenshots, but there are so many, and it's a
huge, disorganised mess.

I created this tool to sort and convert existing screenshots however you prefer and automatically
sort and convert new ones.

## Usage

`./screenshot_organiser <config file>`

If you don't specify a config file, it will try to read `config.json` from the current directory.

## Annotated example config

```javascript
{
  "options": {
    // Where your screenshots are located.
    //
    // This is the base directory for everything this program does. Reshade and FFXIV should both
    // output their screenshots here. This folder is the folder that will be sorted and watched for
    // new screenshots.
    "screenshots_dir": "D:\\FFXIV Screenshots",
    // Regular expressions to match on.
    //
    // If a file in the top level of screenshots_dir matches one of these, it will be run through
    // the pipeline. Every regular expression in this array must have six named groups: year, month,
    // day, hour, minute, second.
    "match": [
      "ffxiv_(?P<month>\\d{2})(?P<day>\\d{2})(?P<year>\\d{4})_(?P<hour>\\d{2})(?P<minute>\\d{2})(?P<second>\\d{2}).(?:jpg|png)",
      "ffxiv_dx11 (?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2}) (?P<hour>\\d{2})-(?P<minute>\\d{2})-(?P<second>\\d{2}).(?:png|bmp)"
    ],
    // How long, in milliseconds, to wait before sending FS events. You should probably not touch
    // this.
    //
    // To ensure files are fully written before being run through the pipeline, there is a delay
    // before checking them. Increase this by increments of 1,000 if you notice your screenshots are
    // being cut off.
    "event_delay": 1000
  },
  // The meat of the application: the pipeline.
  //
  // The pipeline defines jobs to run on each screenshot. The jobs are run in order and can be
  // specified in any order or omitted. If this array is empty, the program will do nothing.
  //
  // There are currently only two jobs: move and convert.
  //
  // The convert job takes whatever screenshot it finds and converts it to whichever format you
  // choose. In the configuration below, all screenshots will be converted to JPGs with a quality of
  // 90/100. This is a pretty safe default.
  //
  // The quality configuration option is only used with JPG. Supported files types are PNG, JPG,
  // GIF, BMP, and ICO. Note that they must be specified in lowercase.
  //
  // There is one more option that can be specified for the convert job. keep_original allows you to
  // keep the original file. Note that any move jobs done will move both the converted and original
  // file if the original is kept. This defaults to off.
  //
  // The move job is very simple. All it does is move the screenshot to whatever path you would
  // like, relative to screenshots_dir.
  //
  // The path is specified using a time format string. Please see
  // https://docs.rs/chrono/0.4.5/chrono/format/strftime/index.html for more information on what
  // special symbols can be used.
  //
  // In the example below, the file is moved into several subdirectories. They will be created
  // automatically if they do not exist.
  //
  // Note that local is set to true in this configuration. If set to false, the timestamp will be
  // UTC. You probably want to keep it set to true. The default is true.
  "pipeline": [
    {
      "job": "convert",
      "options": {
        "to": {
          "format": "jpg",
          "quality": 90
        }
        // "keep_original": true
      }
    },
    {
      "job": "move",
      "options": {
        "to": "%Y\\%m\\%d\\ffxiv.%Y-%m-%d.%H-%M-%S",
        "local": true
      }
    }
  ]
}
```
