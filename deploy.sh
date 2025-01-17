#!/bin/sh

rm -r public ; \
zola build && \
git checkout master && \
cp -r public/* . && \
git add . && \
git commit -m "Site updated at $(date -u --rfc-3339=seconds)" #&& \
git push origin master && \
git checkout zola
