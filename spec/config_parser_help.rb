require 'spec_helper'
require 'monzo/config_parser'

describe '# Config Parser' do

  context 'URL validity' do
    it 'URL should be valid' do

      # Given
      url = 'https://github.com/cesarferreira/android-helloworld'

      # When
      github = Dryrun::Github.new(url)
      expected = 'https://github.com/cesarferreira/android-helloworld.git'

      # Then
      expect(github.clonable_url).to eq(expected)

    end
  end
end
