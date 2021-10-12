# frozen_string_literal: true

require "the-insecure-proxy/streaming_body_rewriter"

RSpec.describe TheInsecureProxy::StreamingBodyRewriter do
  subject(:instance) { described_class.new(Array(body)) }

  describe "#each" do
    subject do
      data = "".dup
      instance.each { |chunk| data << chunk }
      data
    end

    context "when body is just an https://" do
      let(:body) { "https://" }

      it { is_expected.to eq "http://" }
    end

    context "when body has 'hello hto' in two chunks" do
      let(:body) { ["h", "ello hto " ] }

      it { is_expected.to eq "hello hto " }
    end

    context "when body has https:// split across three chunks" do
      let(:body) { ["ht", "tp", "s://" ] }

      it { is_expected.to eq "http://" }
    end

    context "when body has many chunks with https:// spread across the divide" do
      let(:body) { ["h", "ello ht", "o ht", "tyou htt", "pon https", "this http", "s", ":", "lovely https:/day https://website.club - htt", "ps://example.com", " - h","t","t","p","s",":","/","/","telynz.uk"] }

      it "yields rewritten chunks" do
        expect(subject).to eq "hello hto httyou httpon httpsthis https:lovely https:/day http://website.club - http://example.com - http://telynz.uk"
      end
    end
  end
end
