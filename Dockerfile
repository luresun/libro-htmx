# مرحله اول: بیلد و کامپایل کدها با استفاده از محیط آلپاین سبک
FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

# کامپایل پروژه در حالت Release
RUN cargo build --release

# مرحله دوم: آماده‌سازی ایمیج نهایی بسیار سبک برای اجرا
FROM alpine:3.20

WORKDIR /app

# کپی فایل باینری کامپایل‌شده (نام your_project_name را با نام پروژه‌تان در Cargo.toml جایگزین کنید)
COPY --from=builder /app/target/release/your_project_name /app/server

# کپی پوشه‌های قالب‌های Askama و فایل‌های CSS/استاتیک
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/assets /app/assets

# پورت اجرایی سرور
ENV PORT=3000
EXPOSE 3000

CMD ["/app/server"]
