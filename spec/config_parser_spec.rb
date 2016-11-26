require 'spec_helper'
require 'monzo/config_parser'

describe '# Config Parser' do

  it '# The file is present' do

    # Given
    path = 'spec/assets/test_config.yml'

    # When
    config = ConfigParser.new(path)

    # Then
    expect(config.parse).to_not be_nil
    expect(config.parse['access_token']).to eq('Qnjdas8hakxdjasQscGVgnVGIVXpvpZ5uCxkQ5XLnDHnOPoBtXreQ6adBo')
    expect(config.parse['account_id']).to eq('acc_0aksdaklsjSh28181')
    expect(config.parse['user_id']).to eq('user_18231092askdas9212')

  end

  it '# The file is not present' do

    # Given
    path = 'spec/assets/NOT_CORRECT/test_config.yml'

    # When
    config = ConfigParser.new(path)

    # Then
    expect(config.parse).to be_nil

  end

  it '# The file is present but is corrupt' do

    # Given
    path = 'spec/assets/bad_test_config.yml'

    # When
    config = ConfigParser.new(path)

    # Then
    expect(config.parse).to be_nil

  end

end
