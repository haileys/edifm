require "active_record"
require "sinatra"
require_relative "lib/models"

ActiveRecord::Base.establish_connection(ENV.fetch("DATABASE_URL"))

EDIFM_DOMAIN = ENV.fetch("EDIFM_DOMAIN")

get "/" do
  @play = Play.latest
  erb :index
end

get "/_playing" do
  @play = Play.latest
  erb :_playing
end
