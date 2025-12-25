#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Ágora Marketplace - Contrato de Reportes
///
/// Este contrato proporciona funcionalidades de solo lectura para generar
/// reportes y estadísticas a partir de los datos del contrato Marketplace.
///
/// ## Funcionalidades
/// - Top 5 vendedores con mejor reputación
/// - Top 5 compradores con mejor reputación
/// - Productos más vendidos
/// - Estadísticas por categoría
/// - Cantidad de órdenes por usuario
///
/// ## Nota importante
/// Este contrato es de solo lectura y no puede modificar el estado del Marketplace.
#[ink::contract]
mod reportes {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use scale::{Decode, Encode};

    use market::{Estado, MarketplaceRef};

    /// Representa un usuario con su reputación calculada.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct UsuarioConReputacion {
        /// La cuenta del usuario.
        pub usuario: AccountId,
        /// Promedio de reputación multiplicado por 100 para evitar decimales.
        /// Ejemplo: 450 = 4.50 estrellas
        pub promedio_x100: u32,
        /// Cantidad de calificaciones recibidas.
        pub cantidad_calificaciones: u32,
    }

    /// Representa un producto con su cantidad total vendida.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ProductoVendido {
        /// El ID del producto.
        pub id_producto: u32,
        /// El nombre del producto.
        pub nombre: String,
        /// La categoría del producto.
        pub categoria: String,
        /// El vendedor del producto.
        pub vendedor: AccountId,
        /// Cantidad total de unidades vendidas.
        pub unidades_vendidas: u32,
    }

    /// Estadísticas agregadas por categoría.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct EstadisticasCategoria {
        /// Nombre de la categoría.
        pub categoria: String,
        /// Total de ventas (órdenes completadas) en esta categoría.
        pub total_ventas: u32,
        /// Total de unidades vendidas en esta categoría.
        pub total_unidades: u32,
        /// Promedio de calificación de vendedores en esta categoría (x100).
        pub calificacion_promedio_x100: u32,
        /// Cantidad de productos publicados en esta categoría.
        pub cantidad_productos: u32,
    }

    /// Información sobre las órdenes de un usuario.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OrdenesUsuario {
        /// La cuenta del usuario.
        pub usuario: AccountId,
        /// Cantidad de órdenes como comprador.
        pub ordenes_como_comprador: u32,
        /// Cantidad de órdenes como vendedor.
        pub ordenes_como_vendedor: u32,
        /// Cantidad de órdenes completadas como comprador.
        pub completadas_como_comprador: u32,
        /// Cantidad de órdenes completadas como vendedor.
        pub completadas_como_vendedor: u32,
    }

    /// Errores posibles del contrato de reportes.
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// La categoría especificada no existe.
        CategoriaNoEncontrada,
    }

    #[ink(storage)]
    pub struct Reportes {
        marketplace_address: AccountId,
    }

    impl Reportes {
        /// Crea una nueva instancia del contrato de Reportes.
        ///
        /// # Argumentos
        ///
        /// * `marketplace_address` - La dirección del contrato Marketplace del cual se leerán los datos.
        ///
        /// # Nota
        ///
        /// Este contrato es de solo lectura y requiere que el contrato Marketplace
        /// esté desplegado previamente en la red.
        #[ink(constructor)]
        pub fn new(marketplace_address: AccountId) -> Self {
            Self {
                marketplace_address,
            }
        }

        /// Obtiene la dirección del contrato Marketplace asociado.
        ///
        /// # Retorno
        ///
        /// Devuelve el `AccountId` del contrato Marketplace del cual se obtienen los datos.
        #[ink(message)]
        pub fn get_marketplace(&self) -> AccountId {
            self.marketplace_address
        }

        /// Obtiene el top N de vendedores con mejor reputación.
        ///
        /// # Argumentos
        ///
        /// * `limite` - Cantidad máxima de vendedores a retornar (ej: 5 para top 5).
        ///
        /// # Retorno
        ///
        /// Lista ordenada de vendedores por reputación descendente.
        /// En caso de empate en promedio, se ordena por cantidad de calificaciones.
        ///
        /// # Nota
        ///
        /// Solo incluye vendedores que tienen al menos una calificación.
        #[ink(message)]
        pub fn top_vendedores(&self, limite: u32) -> Vec<UsuarioConReputacion> {
            self._top_vendedores(limite)
        }

        /// Obtiene el top N de compradores con mejor reputación.
        ///
        /// # Argumentos
        ///
        /// * `limite` - Cantidad máxima de compradores a retornar (ej: 5 para top 5).
        ///
        /// # Retorno
        ///
        /// Lista ordenada de compradores por reputación descendente.
        /// En caso de empate en promedio, se ordena por cantidad de calificaciones.
        ///
        /// # Nota
        ///
        /// Solo incluye compradores que tienen al menos una calificación.
        #[ink(message)]
        pub fn top_compradores(&self, limite: u32) -> Vec<UsuarioConReputacion> {
            self._top_compradores(limite)
        }

        /// Obtiene los productos más vendidos del marketplace.
        ///
        /// # Argumentos
        ///
        /// * `limite` - Cantidad máxima de productos a retornar.
        ///
        /// # Retorno
        ///
        /// Lista de productos ordenada por unidades vendidas (descendente).
        /// Incluye información del producto, categoría y vendedor.
        ///
        /// # Nota
        ///
        /// Se consideran todas las órdenes excepto las canceladas.
        #[ink(message)]
        pub fn productos_mas_vendidos(&self, limite: u32) -> Vec<ProductoVendido> {
            self._productos_mas_vendidos(limite)
        }

        /// Obtiene estadísticas agregadas de todas las categorías.
        ///
        /// # Retorno
        ///
        /// Lista de estadísticas por cada categoría existente, incluyendo:
        /// - Total de ventas completadas
        /// - Total de unidades vendidas
        /// - Calificación promedio de vendedores (x100)
        /// - Cantidad de productos publicados
        ///
        /// # Nota
        ///
        /// Solo se consideran órdenes en estado `Recibido` para las ventas.
        #[ink(message)]
        pub fn estadisticas_por_categoria(&self) -> Vec<EstadisticasCategoria> {
            self._estadisticas_por_categoria()
        }

        /// Obtiene las estadísticas de una categoría específica.
        ///
        /// # Argumentos
        ///
        /// * `categoria` - Nombre exacto de la categoría a consultar.
        ///
        /// # Retorno
        ///
        /// - `Ok(EstadisticasCategoria)` con las estadísticas de la categoría.
        /// - `Err(Error::CategoriaNoEncontrada)` si la categoría no existe.
        #[ink(message)]
        pub fn estadisticas_categoria(
            &self,
            categoria: String,
        ) -> Result<EstadisticasCategoria, Error> {
            self._estadisticas_categoria(categoria)
        }

        /// Obtiene el conteo de órdenes de un usuario específico.
        ///
        /// # Argumentos
        ///
        /// * `usuario` - La cuenta del usuario a consultar.
        ///
        /// # Retorno
        ///
        /// Estructura con el conteo de órdenes como comprador y vendedor,
        /// tanto totales como completadas.
        #[ink(message)]
        pub fn ordenes_por_usuario(&self, usuario: AccountId) -> OrdenesUsuario {
            self._ordenes_por_usuario(usuario)
        }

        /// Obtiene un resumen de órdenes para todos los usuarios activos.
        ///
        /// # Retorno
        ///
        /// Lista de usuarios con sus conteos de órdenes.
        /// Solo incluye usuarios que tienen al menos una orden.
        #[ink(message)]
        pub fn resumen_ordenes_todos_usuarios(&self) -> Vec<OrdenesUsuario> {
            self._resumen_ordenes_todos_usuarios()
        }

        /// Obtiene un resumen general del marketplace.
        ///
        /// # Retorno
        ///
        /// Tupla con los siguientes valores:
        /// - `0`: Total de usuarios registrados
        /// - `1`: Total de productos publicados
        /// - `2`: Total de órdenes creadas
        /// - `3`: Total de órdenes completadas (estado Recibido)
        #[ink(message)]
        pub fn resumen_general(&self) -> (u32, u32, u32, u32) {
            self._resumen_general()
        }

        /// Obtiene todas las categorías disponibles en el marketplace.
        ///
        /// # Retorno
        ///
        /// Lista de nombres de categorías únicas extraídas de los productos publicados.
        #[ink(message)]
        pub fn listar_categorias(&self) -> Vec<String> {
            self._listar_categorias()
        }

        /// Crea una referencia al contrato Marketplace.
        fn marketplace(&self) -> MarketplaceRef {
            ink::env::call::FromAccountId::from_account_id(self.marketplace_address)
        }

        /// Lógica interna para calcular el top de vendedores.
        ///
        /// # Optimización
        /// Utiliza `listar_todas_reputaciones` para obtener todos los datos en una sola llamada
        /// externa (O(1) llamadas de red), en lugar de iterar y llamar por cada usuario (O(N)).
        /// El filtrado y ordenamiento se realizan localmente en memoria.
        #[allow(clippy::arithmetic_side_effects)]
        fn _top_vendedores(&self, limite: u32) -> Vec<UsuarioConReputacion> {
            let marketplace = self.marketplace();
            let reputaciones = marketplace.listar_todas_reputaciones();

            let mut resultado: Vec<UsuarioConReputacion> = reputaciones
                .into_iter()
                .filter_map(|(usuario, rep)| {
                    let (suma, cantidad) = rep.como_vendedor;
                    if cantidad > 0 {
                        let promedio_x100 = suma.saturating_mul(100).saturating_div(cantidad);
                        Some(UsuarioConReputacion {
                            usuario,
                            promedio_x100,
                            cantidad_calificaciones: cantidad,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            self._ordenar_por_reputacion(&mut resultado);
            resultado.truncate(limite as usize);
            resultado
        }

        /// Lógica interna para calcular el top de compradores.
        ///
        /// # Optimización
        /// Utiliza `listar_todas_reputaciones` para obtener todos los datos en una sola llamada
        /// externa (O(1) llamadas de red), en lugar de iterar y llamar por cada usuario (O(N)).
        /// El filtrado y ordenamiento se realizan localmente en memoria.
        #[allow(clippy::arithmetic_side_effects)]
        fn _top_compradores(&self, limite: u32) -> Vec<UsuarioConReputacion> {
            let marketplace = self.marketplace();
            let reputaciones = marketplace.listar_todas_reputaciones();

            let mut resultado: Vec<UsuarioConReputacion> = reputaciones
                .into_iter()
                .filter_map(|(usuario, rep)| {
                    let (suma, cantidad) = rep.como_comprador;
                    if cantidad > 0 {
                        let promedio_x100 = suma.saturating_mul(100).saturating_div(cantidad);
                        Some(UsuarioConReputacion {
                            usuario,
                            promedio_x100,
                            cantidad_calificaciones: cantidad,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            self._ordenar_por_reputacion(&mut resultado);
            resultado.truncate(limite as usize);
            resultado
        }

        /// Lógica interna para productos más vendidos.
        ///
        /// Complejidad: O(o + p) donde o = cantidad de órdenes y p = cantidad de productos.
        fn _productos_mas_vendidos(&self, limite: u32) -> Vec<ProductoVendido> {
            let marketplace = self.marketplace();
            let ordenes = marketplace.listar_todas_ordenes();
            let productos = marketplace.listar_todos_productos();

            let mut ventas: Vec<(u32, u32)> = Vec::new();

            for (_oid, orden) in &ordenes {
                if orden.estado == Estado::Recibido {
                    if let Some(pos) = ventas.iter().position(|(id, _)| *id == orden.id_prod) {
                        ventas[pos].1 = ventas[pos].1.saturating_add(orden.cantidad);
                    } else {
                        ventas.push((orden.id_prod, orden.cantidad));
                    }
                }
            }

            ventas.sort_by(|a, b| b.1.cmp(&a.1));

            ventas
                .iter()
                .take(limite as usize)
                .filter_map(|(id_prod, unidades)| {
                    productos
                        .iter()
                        .find(|(pid, _)| pid == id_prod)
                        .map(|(_, producto)| ProductoVendido {
                            id_producto: *id_prod,
                            nombre: producto.nombre.clone(),
                            categoria: producto.categoria.clone(),
                            vendedor: producto.vendedor,
                            unidades_vendidas: *unidades,
                        })
                })
                .collect()
        }

        /// Lógica interna para estadísticas por categoría.
        ///
        /// Complejidad: O(p + o) donde p = cantidad de productos y o = cantidad de órdenes.
        #[allow(clippy::arithmetic_side_effects)]
        fn _estadisticas_por_categoria(&self) -> Vec<EstadisticasCategoria> {
            let marketplace = self.marketplace();
            let productos = marketplace.listar_todos_productos();
            let ordenes = marketplace.listar_todas_ordenes();

            struct DatosCat {
                categoria: String,
                total_ventas: u32,
                total_unidades: u32,
                suma_calif: u32,
                cant_calif: u32,
                cant_productos: u32,
            }

            let mut categorias: Vec<DatosCat> = Vec::new();

            for (_pid, producto) in &productos {
                let found = categorias
                    .iter_mut()
                    .find(|c| c.categoria == producto.categoria);
                match found {
                    Some(cat) => cat.cant_productos = cat.cant_productos.saturating_add(1),
                    None => categorias.push(DatosCat {
                        categoria: producto.categoria.clone(),
                        total_ventas: 0,
                        total_unidades: 0,
                        suma_calif: 0,
                        cant_calif: 0,
                        cant_productos: 1,
                    }),
                }
            }

            for (_oid, orden) in &ordenes {
                if orden.estado == Estado::Recibido {
                    if let Some(producto) = productos
                        .iter()
                        .find(|(pid, _)| *pid == orden.id_prod)
                        .map(|(_, p)| p)
                    {
                        if let Some(cat) = categorias
                            .iter_mut()
                            .find(|c| c.categoria == producto.categoria)
                        {
                            cat.total_ventas = cat.total_ventas.saturating_add(1);
                            cat.total_unidades = cat.total_unidades.saturating_add(orden.cantidad);
                        }
                    }
                }
            }

            for cat in categorias.iter_mut() {
                if let Some((suma, cant)) =
                    marketplace.obtener_calificacion_categoria(cat.categoria.clone())
                {
                    cat.suma_calif = suma;
                    cat.cant_calif = cant;
                }
            }

            categorias
                .into_iter()
                .map(|cat| {
                    let suma = cat.suma_calif;
                    let cantidad = cat.cant_calif;
                    let promedio = if cantidad > 0 {
                        suma.saturating_mul(100).saturating_div(cantidad)
                    } else {
                        0
                    };

                    EstadisticasCategoria {
                        categoria: cat.categoria,
                        total_ventas: cat.total_ventas,
                        total_unidades: cat.total_unidades,
                        calificacion_promedio_x100: promedio,
                        cantidad_productos: cat.cant_productos,
                    }
                })
                .collect()
        }

        /// Lógica interna para estadísticas de una categoría específica.
        #[allow(clippy::arithmetic_side_effects)]
        fn _estadisticas_categoria(
            &self,
            categoria: String,
        ) -> Result<EstadisticasCategoria, Error> {
            let marketplace = self.marketplace();
            let productos = marketplace.listar_todos_productos();
            let ordenes = marketplace.listar_todas_ordenes();

            let mut cantidad_productos: u32 = 0;
            let mut total_ventas: u32 = 0;
            let mut total_unidades: u32 = 0;

            for (_pid, producto) in &productos {
                if producto.categoria == categoria {
                    cantidad_productos = cantidad_productos.saturating_add(1);
                }
            }

            if cantidad_productos == 0 {
                return Err(Error::CategoriaNoEncontrada);
            }

            for (_oid, orden) in &ordenes {
                if orden.estado == Estado::Recibido {
                    if let Some(producto) = productos
                        .iter()
                        .find(|(pid, _)| *pid == orden.id_prod)
                        .map(|(_, p)| p)
                    {
                        if producto.categoria == categoria {
                            total_ventas = total_ventas.saturating_add(1);
                            total_unidades = total_unidades.saturating_add(orden.cantidad);
                        }
                    }
                }
            }

            let (suma_calif, cant_calif) = marketplace
                .obtener_calificacion_categoria(categoria.clone())
                .unwrap_or((0, 0));

            let calificacion_promedio_x100 = if cant_calif > 0 {
                suma_calif.saturating_mul(100).saturating_div(cant_calif)
            } else {
                0
            };

            Ok(EstadisticasCategoria {
                categoria,
                total_ventas,
                total_unidades,
                calificacion_promedio_x100,
                cantidad_productos,
            })
        }

        /// Lógica interna para órdenes por usuario.
        ///
        /// Complejidad: O(o) donde o = cantidad de órdenes totales.
        fn _ordenes_por_usuario(&self, usuario: AccountId) -> OrdenesUsuario {
            let marketplace = self.marketplace();
            let ordenes = marketplace.listar_todas_ordenes();

            let mut resultado = OrdenesUsuario {
                usuario,
                ordenes_como_comprador: 0,
                ordenes_como_vendedor: 0,
                completadas_como_comprador: 0,
                completadas_como_vendedor: 0,
            };

            for (_oid, orden) in ordenes {
                if orden.comprador == usuario {
                    resultado.ordenes_como_comprador =
                        resultado.ordenes_como_comprador.saturating_add(1);
                    if orden.estado == Estado::Recibido {
                        resultado.completadas_como_comprador =
                            resultado.completadas_como_comprador.saturating_add(1);
                    }
                }
                if orden.vendedor == usuario {
                    resultado.ordenes_como_vendedor =
                        resultado.ordenes_como_vendedor.saturating_add(1);
                    if orden.estado == Estado::Recibido {
                        resultado.completadas_como_vendedor =
                            resultado.completadas_como_vendedor.saturating_add(1);
                    }
                }
            }

            resultado
        }

        /// Lógica interna para resumen de órdenes de todos los usuarios.
        ///
        /// Complejidad: O(u * o) donde u = cantidad de usuarios y o = cantidad de órdenes.
        fn _resumen_ordenes_todos_usuarios(&self) -> Vec<OrdenesUsuario> {
            let marketplace = self.marketplace();
            let usuarios = marketplace.listar_usuarios();
            let ordenes = marketplace.listar_todas_ordenes();

            let mut resultado: Vec<OrdenesUsuario> = Vec::new();

            for usuario in usuarios {
                let mut info = OrdenesUsuario {
                    usuario,
                    ordenes_como_comprador: 0,
                    ordenes_como_vendedor: 0,
                    completadas_como_comprador: 0,
                    completadas_como_vendedor: 0,
                };

                for (_oid, orden) in &ordenes {
                    if orden.comprador == usuario {
                        info.ordenes_como_comprador = info.ordenes_como_comprador.saturating_add(1);
                        if orden.estado == Estado::Recibido {
                            info.completadas_como_comprador =
                                info.completadas_como_comprador.saturating_add(1);
                        }
                    }
                    if orden.vendedor == usuario {
                        info.ordenes_como_vendedor = info.ordenes_como_vendedor.saturating_add(1);
                        if orden.estado == Estado::Recibido {
                            info.completadas_como_vendedor =
                                info.completadas_como_vendedor.saturating_add(1);
                        }
                    }
                }

                let tiene_ordenes =
                    info.ordenes_como_comprador > 0 || info.ordenes_como_vendedor > 0;

                if tiene_ordenes {
                    resultado.push(info);
                }
            }

            resultado
        }

        /// Lógica interna para resumen general.
        ///
        /// Retorna: (total_usuarios, total_productos, total_ordenes, ordenes_completadas).
        /// Complejidad: O(o) donde o = cantidad de órdenes.
        fn _resumen_general(&self) -> (u32, u32, u32, u32) {
            let marketplace = self.marketplace();
            let usuarios = marketplace.listar_usuarios();
            let productos = marketplace.listar_todos_productos();
            let ordenes = marketplace.listar_todas_ordenes();

            let mut completadas: u32 = 0;
            for (_oid, orden) in &ordenes {
                if orden.estado == Estado::Recibido {
                    completadas = completadas.saturating_add(1);
                }
            }

            (
                u32::try_from(usuarios.len()).unwrap_or(u32::MAX),
                u32::try_from(productos.len()).unwrap_or(u32::MAX),
                u32::try_from(ordenes.len()).unwrap_or(u32::MAX),
                completadas,
            )
        }

        /// Lógica interna para listar categorías únicas.
        ///
        /// Complejidad: O(p * c) donde p = cantidad de productos y c = categorías únicas.
        fn _listar_categorias(&self) -> Vec<String> {
            let marketplace = self.marketplace();
            let productos = marketplace.listar_todos_productos();
            let mut categorias: Vec<String> = Vec::new();

            for (_pid, producto) in productos {
                if !categorias.iter().any(|c| c == &producto.categoria) {
                    categorias.push(producto.categoria);
                }
            }

            categorias
        }

        /// Ordena usuarios por reputación descendente.
        ///
        /// Criterio: primero por promedio (mayor mejor), luego por cantidad de calificaciones.
        /// Complejidad: O(n log n) donde n = cantidad de usuarios.
        fn _ordenar_por_reputacion(&self, usuarios: &mut [UsuarioConReputacion]) {
            usuarios.sort_by(|a, b| {
                if b.promedio_x100 != a.promedio_x100 {
                    b.promedio_x100.cmp(&a.promedio_x100)
                } else {
                    b.cantidad_calificaciones.cmp(&a.cantidad_calificaciones)
                }
            });
        }
    }

    #[cfg(test)]
    include!("unit_tests.rs");
}

#[cfg(any(feature = "ink-as-dependency", feature = "e2e-tests"))]
pub use reportes::{
    Error, EstadisticasCategoria, OrdenesUsuario, ProductoVendido, Reportes, ReportesRef,
    UsuarioConReputacion,
};
