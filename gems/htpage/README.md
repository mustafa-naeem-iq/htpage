# htpage Ruby Gem

High-performance native in-process compiler and Rails template handler for the **htpage** unified markup language, powered by a pure Rust core (`htpage_core`).

## Installation

Add this line to your application's `Gemfile`:

```ruby
gem "htpage"
```

And then execute:

```bash
bundle install
```

## Usage in Ruby on Rails

`htpage` automatically registers itself as a template handler for `ActionView`. Simply name your view templates with `.html.htpage`:

```
app/views/home/index.html.htpage
```

### Example Template

```
div {
  h1 { Welcome to Rails with htpage }
  p { Fast, clean, unified markup. }
}
```

## Standalone Ruby Usage

```ruby
require "htpage"

html = Htpage.compile("h1 { Hello world }")
puts html
# => "<h1>Hello world</h1>\n"
```

## License

MIT © [Mustafa Naeem](https://github.com/mustafa-naeem-iq)
