FROM node:16

WORKDIR /app

COPY package*.json ./

ENV NODE_ENV=production

RUN npm ci

COPY . .

EXPOSE 3000

CMD ["node", "src/index.js"]
