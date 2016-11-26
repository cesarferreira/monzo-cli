# Ensure we require the local version and not one we might have installed already
require File.join([File.dirname(__FILE__), 'lib', 'monzo', 'version.rb'])
spec = Gem::Specification.new do |s|
  s.name = 'monzo-cli'
  s.version = Monzo::VERSION
  s.author = 'cesar ferreira'
  s.email = 'cesar.manuel.ferreira@gmail.com'
  s.homepage = 'http://cesarferreira.com'
  s.platform = Gem::Platform::RUBY
  s.summary = 'A description of your project'
  s.files = `git ls-files`.split("
")
  s.require_paths << 'lib'
  s.required_ruby_version = '>= 2.0.0'
  s.bindir = 'bin'
  s.executables << 'monzo-cli'

  s.add_development_dependency 'rake'
  s.add_development_dependency 'aruba'
  s.add_development_dependency 'rspec'
  s.add_development_dependency 'pry'

  s.add_runtime_dependency 'gli', '2.14.0'
  s.add_runtime_dependency 'terminal-table', '>= 1.7.3'
  # s.add_runtime_dependency 'mondo', '~> 0.5.0'
  s.add_runtime_dependency 'colorize', '~> 0.7'



  s.add_runtime_dependency 'oauth2', '~> 1.0'
  s.add_runtime_dependency 'money'
  s.add_runtime_dependency 'multi_json', '~> 1.10'
  s.add_runtime_dependency 'activesupport', '~> 3.2'


end
