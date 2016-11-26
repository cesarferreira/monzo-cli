class MonzoApi

  def initialize(config)
    @user_id = config.parse['user_id']
    @account_id = config.parse['account_id']
    @access_token = config.parse['access_token']


  end

  def initialize_monzo_api

  end
end
