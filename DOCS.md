# Documentación de Ágora Marketplace

## Contrato Marketplace

Esta sección del documento describe la funcionalidad y el uso del smart contract `Marketplace`, desarrollado en `ink!` como parte del proyecto Ágora Marketplace. El contrato simula un mercado en línea donde los usuarios pueden registrarse como compradores o vendedores, publicar productos y gestionar órdenes de compra.

## Resumen General

El contrato `Marketplace` permite:
1.  **Registro de Usuarios**: Los usuarios se registran con un rol específico: `Comprador`, `Vendedor` o `Ambos`.
2.  **Modificación de Roles**: Los usuarios pueden cambiar su rol después del registro.
3.  **Publicación de Productos**: Los vendedores pueden listar productos con nombre, descripción, precio, stock y categoría.
4.  **Listado de Productos**: Consultar todos los productos de un vendedor específico.
5.  **Compra de Productos**: Los compradores pueden crear órdenes para adquirir productos.
6.  **Listado de Órdenes**: Consultar todas las órdenes de un comprador específico.
7.  **Gestión de Órdenes**: Las órdenes siguen un ciclo de vida simple: `Pendiente` -> `Enviado` -> `Recibido`.

---

## Estructuras de Datos Principales

Estas son las estructuras de datos que definen los elementos clave del marketplace.

### `Rol` (Enum)
Define el papel que un usuario puede tener en el sistema.
- `Comprador`: Solo puede comprar productos.
- `Vendedor`: Solo puede vender productos.
- `Ambos`: Puede comprar y vender.

### `Estado` (Enum)
Representa el estado de una `Orden`.
- `Pendiente`: La orden ha sido creada por el comprador pero el vendedor aún no la ha enviado.
- `Enviado`: El vendedor ha marcado la orden como enviada.
- `Recibido`: El comprador ha confirmado la recepción del producto.

### `Producto` (Struct)
Almacena la información de un artículo en venta.
- `vendedor: AccountId`: La cuenta del usuario que vende el producto.
- `nombre: String`: El nombre del producto (máximo 64 caracteres).
- `descripcion: String`: Descripción detallada del producto (máximo 256 caracteres).
- `precio: Balance`: El costo del producto.
- `stock: u32`: La cantidad de unidades disponibles.
- `categoria: String`: Categoría del producto (máximo 32 caracteres).

### `Orden` (Struct)
Representa una transacción de compra.
- `comprador: AccountId`: La cuenta del usuario que realiza la compra.
- `vendedor: AccountId`: La cuenta del vendedor.
- `id_prod: u32`: El ID del producto comprado.
- `cantidad: u32`: La cantidad de unidades compradas.
- `estado: Estado`: El estado actual de la orden (p. ej., `Pendiente`).

### `Error` (Enum)
Enumera todos los errores que el contrato puede devolver.

| Error              | Descripción                                                                 |
| ------------------ | --------------------------------------------------------------------------- |
| `YaRegistrado`     | El usuario ya tiene un rol asignado y no puede registrarse de nuevo.        |
| `SinRegistro`      | El usuario no está registrado y no puede realizar acciones que lo requieran.|
| `SinPermiso`       | El usuario no tiene el rol adecuado para la acción (p. ej., un comprador intenta vender). |
| `ParamInvalido`    | Uno de los argumentos de la función es inválido (p. ej., precio o stock en 0). |
| `ProdInexistente`  | El ID del producto especificado no existe.                                  |
| `StockInsuf`       | No hay suficientes unidades del producto para completar la compra.          |
| `OrdenInexistente` | El ID de la orden especificada no existe.                                   |
| `EstadoInvalido`   | La orden no está en el estado correcto para la operación (p. ej., marcar como recibido antes de enviado). |
| `IdOverflow`       | Se ha alcanzado el número máximo de productos u órdenes (límite de `u32`).  |

---

## Funciones del Contrato (API)

A continuación se detallan las funciones públicas que pueden ser llamadas por los usuarios.

### `new()`
Crea una nueva instancia del contrato `Marketplace`.
- **Uso**: Se llama una sola vez al desplegar el contrato.

```rust
// Al desplegar el contrato
let marketplace = Marketplace::new();
```

### `registrar(rol: Rol)`
Registra al llamante con el rol especificado.
- **Argumentos**:
  - `rol: Rol`: El rol a asignar (`Comprador`, `Vendedor`, o `Ambos`).
- **Permisos**: Cualquiera puede llamarla, siempre que no esté ya registrado.
- **Errores**: `Error::YaRegistrado`.

**Ejemplo de uso:**
```rust
// Asumimos que 'alice' es la cuenta que llama al contrato.
// Alice se registra como vendedora.
marketplace.registrar(Rol::Vendedor);

// Asumimos que 'bob' es la cuenta que llama al contrato.
// Bob se registra como comprador.
marketplace.registrar(Rol::Comprador);

// Charlie se registra con ambos roles.
marketplace.registrar(Rol::Ambos);
```

### `modificar_rol(nuevo_rol: Rol)`
Modifica el rol de un usuario ya registrado.
- **Argumentos**:
  - `nuevo_rol: Rol`: El nuevo rol a asignar (`Comprador`, `Vendedor`, o `Ambos`).
- **Permisos**: El usuario debe estar previamente registrado.
- **Errores**: `Error::SinRegistro`.

**Ejemplo de uso:**
```rust
// Bob (registrado como Comprador) decide cambiar a Ambos para poder vender.
marketplace.modificar_rol(Rol::Ambos);
assert_eq!(marketplace.obtener_rol(bob), Some(Rol::Ambos));
```

### `obtener_rol(usuario: AccountId)`
Devuelve el rol de una cuenta específica.
- **Retorno**: `Some(Rol)` si el usuario está registrado, `None` en caso contrario.

**Ejemplo de uso:**
```rust
let rol_de_alice = marketplace.obtener_rol(alice_account_id);
// rol_de_alice -> Some(Rol::Vendedor)

let rol_de_charlie = marketplace.obtener_rol(charlie_account_id);
// rol_de_charlie -> None (si no se ha registrado)
```

### `publicar(nombre: String, descripcion: String, precio: Balance, stock: u32, categoria: String)`
Publica un nuevo producto en el marketplace.
- **Argumentos**:
  - `nombre: String`: Nombre del producto (máximo 64 caracteres).
  - `descripcion: String`: Descripción del producto (máximo 256 caracteres).
  - `precio: Balance`: Debe ser mayor que `0`.
  - `stock: u32`: Debe ser mayor que `0`.
  - `categoria: String`: Categoría del producto (máximo 32 caracteres).
- **Permisos**: El llamante debe tener el rol `Vendedor` o `Ambos`.
- **Errores**: `Error::SinPermiso`, `Error::SinRegistro`, `Error::ParamInvalido`, `Error::IdOverflow`.
- **Retorno**: `Ok(u32)` con el ID del nuevo producto.

**Ejemplo de uso:**
```rust
// Alice (vendedora) publica un producto.
let resultado = marketplace.publicar(
    "Laptop Modelo Z".to_string(),
    "Laptop gaming de alta gama con RTX 4090".to_string(),
    1500,
    10,
    "Electrónica".to_string()
);
// resultado -> Ok(1)
let id_producto = resultado.unwrap();
```

### `obtener_producto(id: u32)`
Devuelve la información de un producto por su ID.
- **Retorno**: `Some(Producto)` si existe, `None` en caso contrario.

**Ejemplo de uso:**
```rust
let producto = marketplace.obtener_producto(1);
// producto -> Some(Producto { 
//   vendedor: alice_account_id, 
//   nombre: "Laptop Modelo Z", 
//   descripcion: "Laptop gaming de alta gama con RTX 4090",
//   precio: 1500, 
//   stock: 10,
//   categoria: "Electrónica"
// })
```

### `listar_productos_de_vendedor(vendedor: AccountId)`
Lista todos los productos publicados por un vendedor específico.
- **Argumentos**:
  - `vendedor: AccountId`: La cuenta del vendedor.
- **Retorno**: `Vec<Producto>` con todos los productos del vendedor (vacío si no tiene productos).

**Ejemplo de uso:**
```rust
let productos_alice = marketplace.listar_productos_de_vendedor(alice_account_id);
// productos_alice -> [Producto { ... }, Producto { ... }, ...]
```

### `comprar(id_prod: u32, cant: u32)`
Crea una orden para comprar una cantidad `cant` de un producto `id_prod`.
- **Argumentos**:
  - `id_prod: u32`: El ID del producto a comprar.
  - `cant: u32`: La cantidad a comprar, debe ser mayor que `0`.
- **Permisos**: El llamante debe tener el rol `Comprador` o `Ambos`.
- **Errores**: `Error::SinPermiso`, `Error::SinRegistro`, `Error::ParamInvalido`, `Error::ProdInexistente`, `Error::StockInsuf`, `Error::IdOverflow`.
- **Retorno**: `Ok(u32)` con el ID de la nueva orden.

**Ejemplo de uso:**
```rust
// Bob (comprador) compra 2 unidades del producto con ID 1.
let resultado_compra = marketplace.comprar(1, 2);
// resultado_compra -> Ok(1)
let id_orden = resultado_compra.unwrap();

// Después de la compra, el stock del producto se actualiza.
let producto_actualizado = marketplace.obtener_producto(1).unwrap();
// producto_actualizado.stock -> 8
```

### `obtener_orden(id: u32)`
Devuelve la información de una orden por su ID.
- **Retorno**: `Some(Orden)` si existe, `None` en caso contrario.

**Ejemplo de uso:**
```rust
let orden = marketplace.obtener_orden(1);
// orden -> Some(Orden { comprador: bob_account_id, vendedor: alice_account_id, id_prod: 1, cantidad: 2, estado: Estado::Pendiente })
```

### `listar_ordenes_de_comprador()`
Lista todas las órdenes realizadas por el usuario que llama esta función.

**Razón de privacidad**: Por seguridad y privacidad, un comprador solo puede ver sus propias órdenes. No existe una función pública que permita listar las órdenes de otros usuarios, ya que el historial de compras es información sensible y privada.

- **Permisos**: Cualquier usuario registrado puede llamarla, pero solo verá sus propias órdenes.
- **Retorno**: `Vec<Orden>` con todas las órdenes del caller (vacío si no tiene órdenes).

**Ejemplo de uso:**
```rust
// Bob (comprador) consulta sus propias órdenes.
// El contrato automáticamente usa la cuenta de Bob como filtro.
let mis_ordenes = marketplace.listar_ordenes_de_comprador();
// mis_ordenes -> [Orden { comprador: bob_account_id, ... }, Orden { comprador: bob_account_id, ... }]

// Si Alice intenta llamar la función, solo verá sus propias órdenes (no las de Bob).
```

### `marcar_enviado(oid: u32)`
Cambia el estado de una orden de `Pendiente` a `Enviado`.
- **Argumentos**:
  - `oid: u32`: El ID de la orden.
- **Permisos**: Solo puede ser llamado por el **vendedor** de esa orden.
- **Errores**: `Error::OrdenInexistente`, `Error::SinPermiso`, `Error::EstadoInvalido`.

**Ejemplo de uso:**
```rust
// Alice (vendedora de la orden 1) la marca como enviada.
let resultado = marketplace.marcar_enviado(1);
// resultado -> Ok(())

// Verificamos el nuevo estado de la orden.
let orden_actualizada = marketplace.obtener_orden(1).unwrap();
// orden_actualizada.estado -> Estado::Enviado
```

### `marcar_recibido(oid: u32)`
Cambia el estado de una orden de `Enviado` a `Recibido`.
- **Argumentos**:
  - `oid: u32`: El ID de la orden.
- **Permisos**: Solo puede ser llamado por el **comprador** de esa orden.
- **Errores**: `Error::OrdenInexistente`, `Error::SinPermiso`, `Error::EstadoInvalido`.

**Ejemplo de uso:**
```rust
// Bob (comprador de la orden 1) la marca como recibida.
let resultado = marketplace.marcar_recibido(1);
// resultado -> Ok(())

// Verificamos el estado final de la orden.
let orden_final = marketplace.obtener_orden(1).unwrap();
// orden_final.estado -> Estado::Recibido
```

---

## Flujo de Uso Completo (Ejemplo Práctico)

Este ejemplo ilustra una interacción completa desde el registro hasta la finalización de una orden.

**Cuentas involucradas:**
- `Alice`: Vendedora
- `Bob`: Comprador

```rust
// Se asume la existencia de un objeto 'marketplace' y de las cuentas 'alice' y 'bob'.

// --- PASO 1: Registro de usuarios ---
// Alice se registra como vendedora.
// (Llamado desde la cuenta de Alice)
marketplace.registrar(Rol::Vendedor);
assert_eq!(marketplace.obtener_rol(alice), Some(Rol::Vendedor));

// Bob se registra como comprador.
// (Llamado desde la cuenta de Bob)
marketplace.registrar(Rol::Comprador);
assert_eq!(marketplace.obtener_rol(bob), Some(Rol::Comprador));


// --- PASO 2: Publicación de un producto ---
// Alice publica un producto.
// (Llamado desde la cuenta de Alice)
let id_producto = marketplace.publicar(
    "Libro de ink!".to_string(),
    "Guía completa de programación en ink! v5".to_string(),
    50,
    20,
    "Libros".to_string()
).unwrap();
assert_eq!(id_producto, 1);

// Verificamos que el producto se creó correctamente.
let producto = marketplace.obtener_producto(1).unwrap();
assert_eq!(producto.nombre, "Libro de ink!");
assert_eq!(producto.descripcion, "Guía completa de programación en ink! v5");
assert_eq!(producto.stock, 20);
assert_eq!(producto.categoria, "Libros");

// Alice puede listar sus productos
let productos_alice = marketplace.listar_productos_de_vendedor(alice);
assert_eq!(productos_alice.len(), 1);


// --- PASO 3: Compra del producto ---
// Bob compra 3 unidades del libro.
// (Llamado desde la cuenta de Bob)
let id_orden = marketplace.comprar(1, 3).unwrap();
assert_eq!(id_orden, 1);

// Verificamos que el stock del producto se redujo.
let producto_actualizado = marketplace.obtener_producto(1).unwrap();
assert_eq!(producto_actualizado.stock, 17);

// Verificamos que la orden se creó en estado Pendiente.
let orden = marketplace.obtener_orden(1).unwrap();
assert_eq!(orden.comprador, bob);
assert_eq!(orden.vendedor, alice);
assert_eq!(orden.cantidad, 3);
assert_eq!(orden.estado, Estado::Pendiente);

// Bob puede listar sus propias órdenes (sin necesidad de pasar su AccountId)
// (Llamado desde la cuenta de Bob)
let mis_ordenes = marketplace.listar_ordenes_de_comprador();
assert_eq!(mis_ordenes.len(), 1);


// --- PASO 4: El vendedor envía la orden ---
// Alice marca la orden como enviada.
// (Llamado desde la cuenta de Alice)
marketplace.marcar_enviado(1).unwrap();

// Verificamos el cambio de estado.
let orden_enviada = marketplace.obtener_orden(1).unwrap();
assert_eq!(orden_enviada.estado, Estado::Enviado);


// --- PASO 5: El comprador recibe la orden ---
// Bob confirma la recepción del producto.
// (Llamado desde la cuenta de Bob)
marketplace.marcar_recibido(1).unwrap();

// Verificamos el estado final de la orden.
let orden_recibida = marketplace.obtener_orden(1).unwrap();
assert_eq!(orden_recibida.estado, Estado::Recibido);

// El flujo ha concluido con éxito.
```

---

## Tests y Validación

El contrato incluye una suite comprehensiva de **35 tests** organizados por funcionalidad:

### Cobertura de Tests

#### Registro de Usuarios (4 tests)
- Registro exitoso con cada rol (`Comprador`, `Vendedor`, `Ambos`)
- Error al registrar usuario ya registrado

#### Modificación de Roles (3 tests)
- Modificación exitosa de `Comprador` a `Ambos`
- Modificación exitosa de `Vendedor` a `Ambos`
- Error al modificar rol sin estar registrado

#### Publicación de Productos (8 tests)
- Publicación exitosa con todos los campos
- Errores de permisos y validación de parámetros
- Validación de longitudes máximas (nombre, descripción, categoría)

#### Listado de Productos (2 tests)
- Listado exitoso de productos por vendedor
- Retorno de vector vacío cuando no hay productos

#### Compra de Productos (6 tests)
- Compra exitosa con actualización de stock
- Errores de permisos, stock insuficiente, producto inexistente

#### Listado de Órdenes (2 tests)
- Listado exitoso de órdenes por comprador
- Retorno de vector vacío cuando no hay órdenes

#### Flujo de Órdenes (7 tests)
- Cambios de estado exitosos
- Errores de permisos y estados inválidos

#### Casos Especiales (3 tests)
- Overflow de IDs de producto y orden
- Rol `Ambos` puede comprar y vender

### Ejecutar Tests

```bash
cd contracts/market
cargo test
```

**Resultado esperado**: 35 tests pasando exitosamente.

---

## Contrato Reportes

El contrato `Reportes` está en desarrollo aún, cuando esté listo la documentación para él se encontrará aquí.