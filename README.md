# 🦀 T24-RS: A Modern Core Banking Platform in Rust

**T24-RS** is a fictional rewrite of **Temenos T24**, reimagined as a **modular microservice platform** using [Rust](https://www.rust-lang.org/) and [Actix Web](https://actix.rs/).  
It demonstrates how a legacy core banking solution could be redesigned for **performance, security, and scalability** in the modern age.

---

## ✨ Features

- ⚡ **High Performance** – Rust + Actix Web for ultra-fast APIs  
- 🧩 **Modular Architecture** – each banking domain is its own service  
- 🔐 **Security-First** – built-in audit trails, RBAC, and compliance hooks  
- 🌍 **Global Ready** – multi-currency ledger, FX support, and international transfers  
- 📊 **Event-Driven** – Bufstream-based posting and reporting pipelines  

---

## 📦 Modules

- **Core Banking Platform**  
  - Ledger Service (double-entry, multi-currency)  
  - Posting Engine (real-time, event-sourced)  

- **Customer & Account Management**  
  - KYC & Onboarding Service  
  - Account Service (savings, current, loan products)  

- **Payments & Transfers**  
  - Domestic Payments (ACH, SEPA)  
  - Cross-Border Payments (SWIFT)  

- **Compliance, Risk & Security**  
  - AML/KYC Monitoring  
  - Credit Risk & Exposure Service  
  - Fraud Detection  

- **Reporting & Analytics**  
  - Customer Statements  
  - Management Information Systems (MIS)  
  - Dashboards & KPIs  

---

## 🛠️ Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/) + [Actix Web](https://actix.rs/)  
- **Database**: PostgreSQL (ledger, accounts), DragonFly (caching), Event Store  
- **Messaging**: Bufstream for events and real-time pipelines  
- **Infra**: Docker, Kubernetes-ready services  
- **Auth**: JWT + OAuth2 with RBAC  

---

## 🚀 Getting Started

### Prerequisites
- Rust (latest stable)  
- Docker & Docker Compose  
- PostgreSQL  
- Bufstream (for event streaming)  

### Run Locally

