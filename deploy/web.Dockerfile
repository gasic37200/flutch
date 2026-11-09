# 빌드 컨텍스트: 저장소 루트. SPA 정적 파일 + nginx 설정을 한 이미지에 담는다
# (서버에 소스·정적 파일을 따로 올리지 않아도 이미지 하나로 프론트 버전이 정해진다).
FROM node:22-alpine AS build
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

FROM nginx:1.27-alpine
RUN rm /etc/nginx/conf.d/default.conf
COPY deploy/nginx/flutch.conf /etc/nginx/conf.d/flutch.conf
COPY --from=build /app/dist /usr/share/nginx/html
# certbot이 인증서를 갱신하면 새 인증서를 읽도록 12시간마다 reload 한다.
CMD ["sh", "-c", "(while :; do sleep 12h; nginx -s reload; done) & exec nginx -g 'daemon off;'"]
