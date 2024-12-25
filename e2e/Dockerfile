FROM mcr.microsoft.com/playwright:v1.49.0-noble

RUN adduser --disabled-password --home /e2e e2e

WORKDIR /e2e

USER e2e

COPY playwright.config.ts package.json .

RUN npm install
