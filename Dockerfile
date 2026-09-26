# استفاده از ایمیج دبیان اسلیم (سازگاری بسیار بالاتر نسبت به آلپاین با کریت‌های وب)
FROM rust:1.80-slim-bookworm AS builder

# نصب پکیج‌های کامپایل و کتابخانه‌های OpenSSL
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# محدود کردن هسته‌های موازی کامپایل برای جلوگیری از اتمام رم (RAM 512MB)
ENV CARGO_BUILD_JOBS=1

# کامپایل پروژه
RUN cargo build --release

# مرحله دوم: ران‌تایم نهایی سبک
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# نام باینری پروژه را با نام دقیق داخل Cargo.toml جایگزین کنید
COPY --from=builder /app/target/release/your_project_name /app/server
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/assets /app/assets

ENV PORT=3000
EXPOSE 3000

CMD ["/app/server"]
