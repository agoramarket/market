# 🛒 Ágora Marketplace

**Marketplace descentralizado tipo MercadoLibre**, construido en **Rust** con **Ink!** sobre **Substrate**, como proyecto final de la materia **Seminario de Lenguajes – Opción Rust**.

---

## ⚠️ Estado del Proyecto

> ⚠️ **Este repositorio contiene la entrega parcial correspondiente al hito obligatorio del 18 de julio.**
> El desarrollo del proyecto continúa, y **las funcionalidades completas de reputación, reportes y disputas aún no están implementadas**.
> La cobertura de tests actual cumple con el mínimo requerido (≥ 85%).

---

## 🚀 Características Implementadas (Diciembre 2025)

* ✅ Registro de usuarios con roles (`Comprador`, `Vendedor`, o `Ambos`)
* ✅ **Modificación de roles** después del registro
* ✅ Publicación de productos con **descripción y categoría** (por `Vendedores`)
* ✅ **Listado de productos por vendedor**
* ✅ Compra de productos (por `Compradores`)
* ✅ **Listado de órdenes por comprador**
* ✅ Gestión de órdenes con los estados:
  * `Pendiente`
  * `Enviado`
  * `Recibido`
* ✅ Validaciones completas de roles, estados y errores esperados
* ✅ **Cobertura de tests: 35 tests atómicos** (muy superior al 85% requerido)
* ✅ Documentación técnica completa en formato estándar de Rust
* ✅ Contrato desplegado en testnet pública (Shibuya)

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

* ✅ **35 tests ejecutados exitosamente** (organizados por funcionalidad)
* 📈 **Cobertura de código: Superior al 85% requerido**
* ✅ Tests atómicos y bien documentados
* ✅ Cobertura completa de casos de éxito y error

---

## 🔐 Funcionalidades Clave

### Gestión de Usuarios

* `registrar(rol)` - Registra un nuevo usuario con rol `Comprador`, `Vendedor` o `Ambos`
* `modificar_rol(nuevo_rol)` - Permite cambiar el rol después del registro
* `obtener_rol(usuario)` - Consulta el rol de un usuario

### Funciones de Vendedor

* `publicar(nombre, descripcion, precio, stock, categoria)` - Publica un producto completo
* `listar_productos_de_vendedor(vendedor)` - Lista todos los productos de un vendedor
* `marcar_enviado(orden_id)` - Marca una orden como enviada

### Funciones de Comprador

* `comprar(producto_id, cantidad)` - Crea una orden de compra
* `listar_ordenes_de_comprador(comprador)` - Lista todas las órdenes de un comprador
* `marcar_recibido(orden_id)` - Confirma la recepción de una orden

### Consultas Generales

* `obtener_producto(id)` - Obtiene los detalles de un producto
* `obtener_orden(id)` - Obtiene los detalles de una orden

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