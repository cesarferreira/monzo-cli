require 'mondo'

class MonzoApi

  def initialize(config)
    @user_id = config.parse['user_id']
    @account_id = config.parse['account_id']
    @access_token = config.parse['access_token']

    client = Mondo::Client.new(token: @access_token, account_id: @user_id)
    puts client.balance @account_id
  end

  def initialize_monzo_api

  end
end
