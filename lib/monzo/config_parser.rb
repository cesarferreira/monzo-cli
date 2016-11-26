require 'yaml'

class ConfigParser

  def initialize(path_to_config)
    @path_to_config = path_to_config
    @parsed = nil

    @parsed = begin
      YAML.load(File.open(@path_to_config))
    rescue ArgumentError => e
      puts "Could not parse YAML: #{e.message}"
    end if File.exist?(@path_to_config)

  end

  def parse
    return nil unless valid?
    @parsed
  end

  def valid?
    return nil if @parsed.nil?
    return nil if @parsed['access_token'].nil?
    return nil if @parsed['account_id'].nil?
    return nil if @parsed['user_id'].nil?

    true
  end

end
