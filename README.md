# 🛒 Ágora Marketplace

**Marketplace descentralizado tipo MercadoLibre**, construido en **Rust** con **Ink!** sobre **Substrate**, como proyecto final de la materia **Seminario de Lenguajes – Opción Rust**.

---

## ⚠️ Estado del Proyecto

> ⚠️ **Este repositorio contiene la entrega parcial correspondiente al hito obligatorio del 18 de julio.**
> El desarrollo del proyecto continúa, y **las funcionalidades completas de reputación, reportes y disputas aún no están implementadas**.
> La cobertura de tests actual cumple con el mínimo requerido (≥ 85%).

---

## 🚀 Características Implementadas (18 de julio)

* ✅ Registro de usuarios con roles (`Comprador`, `Vendedor`, o ambos).
* ✅ Publicación de productos (por `Vendedores`).
* ✅ Compra de productos (por `Compradores`).
* ✅ Gestión de órdenes con los estados:
  * `pendiente`
  * `enviado`
  * `recibido`
* ✅ Validaciones de roles, estados y errores esperados.
* ✅ Cobertura de tests superior al 85%.
* ✅ Documentación interna en formato estándar de Rust.
* ✅ Contrato desplegado en testnet pública (Shibuya).

---

## 📁 Estructura del Proyecto

```
agoramarket/
├── .gitignore
├── LICENSE
├── DOCS.md            ← Documentación técnica interna
├── README.md
└── contracts/
    └── market/
        ├── Cargo.toml
        └── lib.rs     ← Lógica principal del contrato Marketplace
```

---

## ⚙️ Instalación

### Requisitos

* Rust (edición 2021)
* `cargo-contract` (para compilar contratos Ink!)

### Pasos

```bash
# Clonar el repositorio
git clone https://github.com/agoramarket/agoramarket
cd agoramarket/contracts/market

# Instalar herramientas necesarias
cargo install cargo-contract

# Compilar el contrato
cargo contract build
```

---

## 🧪 Tests y Cobertura

```bash
cd contracts/market
cargo test
```

### Resultados

* ✅ 4 tests ejecutados exitosamente
* 📈 **Cobertura de código: 97.44%** (76/78 líneas, tarpaulin)

---

## 🔐 Funcionalidades Clave

### Usuarios

* `registrar(rol)`
* `cambiar_rol(usuario, nuevo_rol)`
* `obtener_rol(usuario)`

### Vendedores

* `publicar_producto(nombre, descripcion, precio, cantidad, categoria)`
* `visualizar_productos_propios()`
* `marcar_enviado(orden_id)`

### Compradores

* `comprar(producto_id, cantidad)`
* `marcar_recibido(orden_id)`
* `cancelar_orden(orden_id)`

---

## 🌐 Contrato en Testnet

* Red: **Astar Shibuya Testnet**
* Dirección del contrato:
  `XDHDTFonKyVQnTZaB9TpMcfTKWkuuL9TaDR4mBz5ebVWnYV`

### Cómo Probar

1. Sigue los pasos para compilar el contrato y obtener los archivos del contrato, entre los cuales está `market.json`, que es la metadata del contrato.
2. Instala la extensión [Polkadot.js](https://polkadot.js.org/extension/)
3. Solicita fondos en el [faucet oficial de Shibuya](https://portal.astar.network/shibuya-testnet/assets)
4. Accede a [https://ui.use.ink](https://ui.use.ink) y carga el contrato usando la dirección on-chain y el `market.json` que compilaste como metadata.
5. Divertite!


---

## 📌 Próximas Etapas (Entrega Final)

* Reputación bidireccional (`Comprador` ↔ `Vendedor`)
* Contrato de reportes (`Reportes`)

  * Top usuarios, productos más vendidos, estadísticas por categoría
* Disputas y simulación de pagos (bonus)
* Refactor y optimización
* Cobertura de tests ≥ 85% en ambos contratos
* Documentación completa y técnica

---

## 📄 Licencia

Este proyecto está bajo la licencia **GPL v3**. Ver [LICENSE](LICENSE) para más detalles.

---

**Desarrollado por The Ágora Developers – 2025** 🚀

---