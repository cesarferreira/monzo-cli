require_relative '../api/mondo'

class MonzoApi

  def initialize(config)
    @user_id = config.parse['user_id']
    @account_id = config.parse['account_id']
    @access_token = config.parse['access_token']

    @monzo = Mondo::Client.new(
        token: @access_token,
        account_id: @user_id)
  end

  def balance
    @monzo.balance @account_id
  end

  def accounts
    @monzo.accounts
  end

  def transactions
    @monzo.transactions
  end

end
