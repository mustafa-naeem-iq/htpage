# frozen_string_literal: true

require_relative "htpage/version"

begin
  require_relative "../../../target/debug/htpage_rb"
rescue LoadError
  begin
    require_relative "../../../target/release/htpage_rb"
  rescue LoadError
    require "htpage_rb"
  end
end

require_relative "htpage/template_handler"

module Htpage
  class Error < StandardError; end
end

if defined?(ActionView::Template)
  ActionView::Template.register_template_handler(:htpage, Htpage::TemplateHandler)
end
