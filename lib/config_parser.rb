require 'yml'

class ConfigParser

   def initialize(path_to_config='~/.monzo-cli.yml')
      @path_to_config = path_to_config
      puts @path_to_config
   end


end
