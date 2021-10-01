FROM ruby:2.7

RUN mkdir /the-insecure-proxy

COPY . /the-insecure-proxy

WORKDIR /the-insecure-proxy

RUN gem install bundler \
 && bundle install \
 && useradd the-insecure-proxy \
 && chown -R the-insecure-proxy .

USER the-insecure-proxy
CMD [ "bundle", "exec", "puma", "-p", "3080" ]
