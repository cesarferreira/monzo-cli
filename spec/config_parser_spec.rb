require 'spec_helper'
require 'monzo/config_parser'

describe '# Config Parser' do

  it '# The file is present' do

    # Given
    path = 'spec/assets/test_config.yml'

    # When
    parser = ConfigParser.new(path)

    # Then
    expect(parser.parse['access_token']).to eq('Qnjdas8hakxdjasQscGVgnVGIVXpvpZ5uCxkQ5XLnDHnOPoBtXreQ6adBo')
    expect(parser.parse['account_id']).to eq('acc_0aksdaklsjSh28181')
    expect(parser.parse['user_id']).to eq('user_18231092askdas9212')

  end

  it '# The file is not present' do

    # Given
    path = 'spec/assets/NOT_CORRECT/test_config.yml'

    # When
    parser = ConfigParser.new(path)

    # Then
    expect(parser.parse).to be_nil

  end

  it '# The file is present but is corrupt' do

    # Given
    path = 'spec/assets/bad_test_config.yml'

    # When
    parser = ConfigParser.new(path)

    # Then
    expect(parser.parse).to be_nil

  end

end
