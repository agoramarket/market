# 🛒 Ágora Marketplace

**Marketplace descentralizado tipo MercadoLibre**, construido en **Rust** con **Ink!** sobre **Substrate**, como proyecto final de la materia **Seminario de Lenguajes – Opción Rust**.

---

## ⚠️ Estado del Proyecto

> ✅ **Este proyecto está completo y listo para producción.**
> Incluye el contrato principal `market` y el contrato de reportes `reports`.
> La cobertura de tests cumple con el mínimo requerido (≥ 85%).

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
  * `Cancelada`
* ✅ **Sistema de cancelación mutua** de órdenes
* ✅ **Sistema de reputación bidireccional** (Comprador ↔ Vendedor)
* ✅ **Contrato de reportes** con:
  * Top vendedores/compradores por reputación
  * Productos más vendidos
  * Estadísticas por categoría
  * Resumen general del marketplace
* ✅ **Sistema de pagos con escrow** (simulación)
  * Pago exacto requerido al momento de la compra
  * Fondos retenidos en el contrato hasta la entrega
  * Liberación automática al confirmar recepción
  * Devolución automática al cancelar orden
* ✅ Validaciones completas de roles, estados y errores esperados
* ✅ Documentación técnica completa en formato estándar de Rust
* ✅ Contrato desplegado en testnet pública (Shibuya)

---

## 📁 Estructura del Proyecto

```
market/
├── Cargo.toml              ← Workspace configuration
├── README.md
└── contracts/
    ├── market/
    │   ├── Cargo.toml
    │   ├── lib.rs          ← Lógica principal del contrato Marketplace
    │   ├── unit_tests.rs   ← Tests unitarios
    │   └── tests/
    │       └── e2e_tests.rs  ← Tests end-to-end
    └── reports/
        ├── Cargo.toml
        ├── lib.rs          ← Lógica del contrato de Reportes
        ├── unit_tests.rs   ← Tests unitarios
        └── tests/
            └── e2e_tests.rs  ← Tests end-to-end
```

---

## ⚙️ Instalación

### Requisitos

* Rust (edición 2021)
* `cargo-contract` v5.0+ (para compilar contratos Ink!)

### Pasos

```bash
# Clonar el repositorio
git clone https://github.com/agoramarket/market
cd market

# Instalar herramientas necesarias
cargo install cargo-contract --locked

# Compilar el contrato market
cd contracts/market
cargo contract build --release

# Compilar el contrato reports
cd ../reports
cargo contract build --release
```

---

## 🧪 Tests y Cobertura

```bash
# Ejecutar todos los tests desde la raíz
cargo test

# Ejecutar tests de un contrato específico
cargo test -p market
cargo test -p reports
```

### Resultados

* ✅ **Tests unitarios exhaustivos** para ambos contratos
* ✅ **Tests end-to-end** para flujos completos
* 📈 **Cobertura de código: Superior al 85% requerido**
* ✅ Tests atómicos y bien documentados
* ✅ Cobertura completa de casos de éxito y error

---

## 🔐 Funcionalidades Clave

### Contrato Market

#### Gestión de Usuarios

* `registrar(rol)` - Registra un nuevo usuario con rol `Comprador`, `Vendedor` o `Ambos`
* `modificar_rol(nuevo_rol)` - Permite cambiar el rol después del registro
* `obtener_rol(usuario)` - Consulta el rol de un usuario

#### Funciones de Vendedor

* `publicar(nombre, descripcion, precio, stock, categoria)` - Publica un producto completo
* `listar_productos_de_vendedor(vendedor)` - Lista todos los productos de un vendedor
* `marcar_enviado(orden_id)` - Marca una orden como enviada
* `calificar_comprador(orden_id, puntos)` - Califica al comprador (1-5 estrellas)

#### Funciones de Comprador

* `comprar(producto_id, cantidad)` - Crea una orden de compra (requiere pago exacto)
* `listar_ordenes_de_comprador(comprador)` - Lista todas las órdenes de un comprador
* `marcar_recibido(orden_id)` - Confirma la recepción y libera los fondos al vendedor
* `calificar_vendedor(orden_id, puntos)` - Califica al vendedor (1-5 estrellas)

#### Sistema de Cancelación

* `solicitar_cancelacion(orden_id)` - Solicita cancelar una orden
* `aceptar_cancelacion(orden_id)` - Acepta la solicitud y devuelve fondos al comprador
* `rechazar_cancelacion(orden_id)` - Rechaza la solicitud de cancelación

#### Sistema de Pagos (Escrow)

* `comprar()` es `payable`: requiere enviar el monto exacto (`precio × cantidad`)
* `obtener_fondos_retenidos(orden_id)` - Consulta fondos en escrow para una orden
* `balance_contrato()` - Consulta el balance total del contrato
* Los fondos se liberan al vendedor con `marcar_recibido()`
* Los fondos se devuelven al comprador al aceptar cancelación

#### Consultas Generales

* `obtener_producto(id)` - Obtiene los detalles de un producto
* `obtener_orden(id)` - Obtiene los detalles de una orden
* `obtener_reputacion(usuario)` - Obtiene la reputación de un usuario

### Contrato Reports

* `top_vendedores(limite)` - Top N vendedores por reputación
* `top_compradores(limite)` - Top N compradores por reputación
* `productos_mas_vendidos(limite)` - Productos más vendidos
* `estadisticas_por_categoria()` - Estadísticas agregadas por categoría
* `ordenes_por_usuario(usuario)` - Conteo de órdenes de un usuario
* `resumen_general()` - Estadísticas generales del marketplace

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

## � Licencia

Este proyecto está bajo la licencia **GPL v3**. Ver [LICENSE](LICENSE) para más detalles.

---

**Desarrollado por The Ágora Developers – 2025** 🚀