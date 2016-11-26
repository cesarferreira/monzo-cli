require 'yaml'

class ConfigParser

  def initialize(path_to_config = '~/.monzo-cli.yml')
    @parsed = nil

    unless File.exist?(path_to_config)
      return
    end
    
    @parsed = begin
      YAML.load(File.open(path_to_config))
    rescue ArgumentError => e
      puts "Could not parse YAML: #{e.message}"
    end
  end

  def parse

    return nil if @parsed.nil?
    return nil if @parsed['access_token'].nil?
    return nil if @parsed['account_id'].nil?
    return nil if @parsed['user_id'].nil?

   @parsed
  end
end
