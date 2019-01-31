class Play < ActiveRecord::Base
  belongs_to :recording
  belongs_to :program

  def self.latest
    order("id DESC").first
  end
end

class Program < ActiveRecord::Base
  has_many :plays
  has_many :program_tags
  has_many :tags, through: :program_tags
end

class ProgramTag < ActiveRecord::Base
  belongs_to :program
  belongs_to :tag
end

class Tag < ActiveRecord::Base
  has_many :program_tags
  has_many :recording_tags
  has_many :programs, through: :program_tags
  has_many :recordings, through: :recording_tags
end

class RecordingTag < ActiveRecord::Base
  belongs_to :tag
  belongs_to :recording
end

class Recording < ActiveRecord::Base
  has_many :recording_tags
  has_many :tags, through: :recording_tags
end
