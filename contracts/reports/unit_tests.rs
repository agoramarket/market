mod tests {
    use super::*;
    use market::{Orden, Producto, ReputacionUsuario};

    /// Crea una cuenta de prueba con un byte específico.
    fn cuenta(n: u8) -> AccountId {
        AccountId::from([n; 32])
    }

    /// Helper para crear un producto de prueba.
    fn crear_producto(vendedor: u8, nombre: &str, categoria: &str, precio: u128) -> Producto {
        Producto {
            vendedor: cuenta(vendedor),
            nombre: String::from(nombre),
            descripcion: String::from("Descripción"),
            precio,
            stock: 10,
            categoria: String::from(categoria),
        }
    }

    /// Helper para crear una orden de prueba.
    fn crear_orden(
        comprador: u8,
        vendedor: u8,
        id_prod: u32,
        cantidad: u32,
        estado: Estado,
    ) -> Orden {
        Orden {
            comprador: cuenta(comprador),
            vendedor: cuenta(vendedor),
            id_prod,
            cantidad,
            estado,
            monto_total: 1000,
        }
    }

    /// Helper para crear una reputación de prueba.
    fn crear_reputacion(como_vendedor: (u32, u32), como_comprador: (u32, u32)) -> ReputacionUsuario {
        ReputacionUsuario {
            como_vendedor,
            como_comprador,
        }
    }

    // ==================== TESTS DE CONSTRUCTOR ====================

    #[ink::test]
    fn test_constructor() {
        let marketplace_addr = cuenta(1);
        let reportes = Reportes::new(marketplace_addr);
        assert_eq!(reportes.get_marketplace(), marketplace_addr);
    }

    #[ink::test]
    fn test_constructor_diferentes_direcciones() {
        let addr1 = cuenta(10);
        let addr2 = cuenta(20);

        let reportes1 = Reportes::new(addr1);
        let reportes2 = Reportes::new(addr2);

        assert_eq!(reportes1.get_marketplace(), addr1);
        assert_eq!(reportes2.get_marketplace(), addr2);
        assert_ne!(reportes1.get_marketplace(), reportes2.get_marketplace());
    }

    // ==================== TESTS DE _ordenar_por_reputacion ====================

    #[ink::test]
    fn test_ordenar_por_reputacion_basico() {
        let mut usuarios = vec![
            UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 300,
                cantidad_calificaciones: 5,
            },
            UsuarioConReputacion {
                usuario: cuenta(3),
                promedio_x100: 500,
                cantidad_calificaciones: 2,
            },
            UsuarioConReputacion {
                usuario: cuenta(4),
                promedio_x100: 500,
                cantidad_calificaciones: 10,
            },
        ];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios[0].usuario, cuenta(4));
        assert_eq!(usuarios[0].promedio_x100, 500);
        assert_eq!(usuarios[0].cantidad_calificaciones, 10);

        assert_eq!(usuarios[1].usuario, cuenta(3));
        assert_eq!(usuarios[1].promedio_x100, 500);

        assert_eq!(usuarios[2].usuario, cuenta(2));
        assert_eq!(usuarios[2].promedio_x100, 300);
    }

    #[ink::test]
    fn test_ordenar_por_reputacion_lista_vacia() {
        let mut usuarios: Vec<UsuarioConReputacion> = Vec::new();
        Reportes::_ordenar_por_reputacion(&mut usuarios);
        assert!(usuarios.is_empty());
    }

    #[ink::test]
    fn test_ordenar_por_reputacion_un_elemento() {
        let mut usuarios = vec![UsuarioConReputacion {
            usuario: cuenta(1),
            promedio_x100: 400,
            cantidad_calificaciones: 5,
        }];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios.len(), 1);
        assert_eq!(usuarios[0].promedio_x100, 400);
    }

    #[ink::test]
    fn test_ordenar_por_reputacion_empate_total() {
        let mut usuarios = vec![
            UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 450,
                cantidad_calificaciones: 10,
            },
            UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 450,
                cantidad_calificaciones: 10,
            },
        ];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios.len(), 2);
        assert_eq!(usuarios[0].promedio_x100, 450);
        assert_eq!(usuarios[1].promedio_x100, 450);
    }

    #[ink::test]
    fn test_ordenar_desempate_por_cantidad_calificaciones() {
        let mut usuarios = vec![
            UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 400,
                cantidad_calificaciones: 5,
            },
            UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 400,
                cantidad_calificaciones: 20,
            },
            UsuarioConReputacion {
                usuario: cuenta(3),
                promedio_x100: 400,
                cantidad_calificaciones: 10,
            },
        ];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios[0].cantidad_calificaciones, 20);
        assert_eq!(usuarios[1].cantidad_calificaciones, 10);
        assert_eq!(usuarios[2].cantidad_calificaciones, 5);
    }

    #[ink::test]
    fn test_ordenar_muchos_usuarios() {
        let mut usuarios = vec![
            UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 100,
                cantidad_calificaciones: 1,
            },
            UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 500,
                cantidad_calificaciones: 100,
            },
            UsuarioConReputacion {
                usuario: cuenta(3),
                promedio_x100: 350,
                cantidad_calificaciones: 50,
            },
            UsuarioConReputacion {
                usuario: cuenta(4),
                promedio_x100: 450,
                cantidad_calificaciones: 25,
            },
            UsuarioConReputacion {
                usuario: cuenta(5),
                promedio_x100: 200,
                cantidad_calificaciones: 10,
            },
        ];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios[0].promedio_x100, 500);
        assert_eq!(usuarios[1].promedio_x100, 450);
        assert_eq!(usuarios[2].promedio_x100, 350);
        assert_eq!(usuarios[3].promedio_x100, 200);
        assert_eq!(usuarios[4].promedio_x100, 100);
    }

    #[ink::test]
    fn test_ordenar_preserva_todos_elementos() {
        let mut usuarios = vec![
            UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 300,
                cantidad_calificaciones: 5,
            },
            UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 400,
                cantidad_calificaciones: 10,
            },
        ];

        let len_original = usuarios.len();
        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios.len(), len_original);
    }

    // ==================== TESTS DE _procesar_top_vendedores ====================

    #[ink::test]
    fn test_procesar_top_vendedores_basico() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((20, 5), (0, 0))),  // promedio 400
            (cuenta(2), crear_reputacion((25, 5), (0, 0))),  // promedio 500
            (cuenta(3), crear_reputacion((15, 5), (0, 0))),  // promedio 300
        ];

        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 5);

        assert_eq!(resultado.len(), 3);
        assert_eq!(resultado[0].usuario, cuenta(2));
        assert_eq!(resultado[0].promedio_x100, 500);
        assert_eq!(resultado[1].usuario, cuenta(1));
        assert_eq!(resultado[2].usuario, cuenta(3));
    }

    #[ink::test]
    fn test_procesar_top_vendedores_con_limite() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((20, 5), (0, 0))),
            (cuenta(2), crear_reputacion((25, 5), (0, 0))),
            (cuenta(3), crear_reputacion((15, 5), (0, 0))),
            (cuenta(4), crear_reputacion((10, 5), (0, 0))),
        ];

        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 2);

        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].promedio_x100, 500);
        assert_eq!(resultado[1].promedio_x100, 400);
    }

    #[ink::test]
    fn test_procesar_top_vendedores_sin_calificaciones() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((0, 0), (10, 2))),
            (cuenta(2), crear_reputacion((0, 0), (15, 3))),
        ];

        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 5);

        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_top_vendedores_mixto() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((20, 4), (10, 2))),
            (cuenta(2), crear_reputacion((0, 0), (15, 3))),
            (cuenta(3), crear_reputacion((12, 3), (5, 1))),
        ];

        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 5);

        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].usuario, cuenta(1));
        assert_eq!(resultado[1].usuario, cuenta(3));
    }

    #[ink::test]
    fn test_procesar_top_vendedores_lista_vacia() {
        let reputaciones: Vec<(AccountId, ReputacionUsuario)> = Vec::new();
        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 5);
        assert!(resultado.is_empty());
    }

    // ==================== TESTS DE _procesar_top_compradores ====================

    #[ink::test]
    fn test_procesar_top_compradores_basico() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((0, 0), (20, 5))),
            (cuenta(2), crear_reputacion((0, 0), (25, 5))),
            (cuenta(3), crear_reputacion((0, 0), (15, 5))),
        ];

        let resultado = Reportes::_procesar_top_compradores(reputaciones, 5);

        assert_eq!(resultado.len(), 3);
        assert_eq!(resultado[0].usuario, cuenta(2));
        assert_eq!(resultado[0].promedio_x100, 500);
    }

    #[ink::test]
    fn test_procesar_top_compradores_con_limite() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((0, 0), (20, 5))),
            (cuenta(2), crear_reputacion((0, 0), (25, 5))),
            (cuenta(3), crear_reputacion((0, 0), (15, 5))),
        ];

        let resultado = Reportes::_procesar_top_compradores(reputaciones, 1);

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].promedio_x100, 500);
    }

    #[ink::test]
    fn test_procesar_top_compradores_sin_calificaciones() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((10, 2), (0, 0))),
            (cuenta(2), crear_reputacion((15, 3), (0, 0))),
        ];

        let resultado = Reportes::_procesar_top_compradores(reputaciones, 5);

        assert!(resultado.is_empty());
    }

    // ==================== TESTS DE _procesar_productos_mas_vendidos ====================

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_basico() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 1, 3, Estado::Recibido)),
            (3, crear_orden(12, 2, 2, 1, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 5);

        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].id_producto, 1);
        assert_eq!(resultado[0].unidades_vendidas, 5);
        assert_eq!(resultado[1].id_producto, 2);
        assert_eq!(resultado[1].unidades_vendidas, 1);
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_ignora_no_recibidos() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 5, Estado::Recibido)),
            (2, crear_orden(11, 1, 1, 3, Estado::Pendiente)),
            (3, crear_orden(12, 1, 1, 2, Estado::Enviado)),
            (4, crear_orden(13, 1, 1, 1, Estado::Cancelada)),
        ];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 5);

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].unidades_vendidas, 5);
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_con_limite() {
        let productos = vec![
            (1, crear_producto(1, "A", "Cat", 100)),
            (2, crear_producto(1, "B", "Cat", 100)),
            (3, crear_producto(1, "C", "Cat", 100)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 10, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 5, Estado::Recibido)),
            (3, crear_orden(12, 1, 3, 3, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 2);

        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].unidades_vendidas, 10);
        assert_eq!(resultado[1].unidades_vendidas, 5);
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_sin_ordenes() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
        ];
        let ordenes: Vec<(u32, Orden)> = Vec::new();

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 5);

        assert!(resultado.is_empty());
    }

    // ==================== TESTS DE _procesar_estadisticas_por_categoria ====================

    #[ink::test]
    fn test_procesar_estadisticas_por_categoria_basico() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
            (3, crear_producto(3, "Libro", "Libros", 20)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 2, 2, 5, Estado::Recibido)),
            (3, crear_orden(12, 3, 3, 1, Estado::Recibido)),
        ];

        let calificaciones = vec![
            (String::from("Electrónica"), (45, 10)),
            (String::from("Libros"), (40, 10)),
        ];

        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, ordenes, calificaciones);

        assert_eq!(resultado.len(), 2);
        
        let electronica = resultado.iter().find(|s| s.categoria == "Electrónica").unwrap();
        assert_eq!(electronica.cantidad_productos, 2);
        assert_eq!(electronica.total_ventas, 2);
        assert_eq!(electronica.total_unidades, 7);
        assert_eq!(electronica.calificacion_promedio_x100, 450);

        let libros = resultado.iter().find(|s| s.categoria == "Libros").unwrap();
        assert_eq!(libros.cantidad_productos, 1);
        assert_eq!(libros.total_ventas, 1);
        assert_eq!(libros.total_unidades, 1);
    }

    #[ink::test]
    fn test_procesar_estadisticas_por_categoria_sin_calificaciones() {
        let productos = vec![
            (1, crear_producto(1, "Test", "NuevaCat", 100)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 3, Estado::Recibido)),
        ];

        let calificaciones: Vec<(String, (u32, u32))> = Vec::new();

        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, ordenes, calificaciones);

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].calificacion_promedio_x100, 0);
    }

    #[ink::test]
    fn test_procesar_estadisticas_por_categoria_sin_ventas() {
        let productos = vec![
            (1, crear_producto(1, "Test", "Cat", 100)),
        ];

        let ordenes: Vec<(u32, Orden)> = Vec::new();
        let calificaciones: Vec<(String, (u32, u32))> = Vec::new();

        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, ordenes, calificaciones);

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].total_ventas, 0);
        assert_eq!(resultado[0].total_unidades, 0);
        assert_eq!(resultado[0].cantidad_productos, 1);
    }

    // ==================== TESTS DE _procesar_estadisticas_categoria ====================

    #[ink::test]
    fn test_procesar_estadisticas_categoria_existente() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 2, 2, 3, Estado::Recibido)),
        ];

        let calificacion = (45, 10);

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos,
            ordenes,
            String::from("Electrónica"),
            calificacion,
        );

        assert!(resultado.is_ok());
        let stats = resultado.unwrap();
        assert_eq!(stats.categoria, "Electrónica");
        assert_eq!(stats.cantidad_productos, 2);
        assert_eq!(stats.total_ventas, 2);
        assert_eq!(stats.total_unidades, 5);
        assert_eq!(stats.calificacion_promedio_x100, 450);
    }

    #[ink::test]
    fn test_procesar_estadisticas_categoria_no_encontrada() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
        ];

        let ordenes: Vec<(u32, Orden)> = Vec::new();
        let calificacion = (0, 0);

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos,
            ordenes,
            String::from("NoExiste"),
            calificacion,
        );

        assert!(resultado.is_err());
        assert_eq!(resultado.unwrap_err(), Error::CategoriaNoEncontrada);
    }

    #[ink::test]
    fn test_procesar_estadisticas_categoria_sin_calificaciones() {
        let productos = vec![
            (1, crear_producto(1, "Test", "Cat", 100)),
        ];

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 5, Estado::Recibido)),
        ];

        let calificacion = (0, 0);

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos,
            ordenes,
            String::from("Cat"),
            calificacion,
        );

        assert!(resultado.is_ok());
        let stats = resultado.unwrap();
        assert_eq!(stats.calificacion_promedio_x100, 0);
    }

    // ==================== TESTS DE _procesar_ordenes_por_usuario ====================

    #[ink::test]
    fn test_procesar_ordenes_por_usuario_como_comprador() {
        let ordenes = vec![
            (1, crear_orden(1, 10, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 11, 2, 3, Estado::Pendiente)),
            (3, crear_orden(1, 12, 3, 1, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));

        assert_eq!(resultado.ordenes_como_comprador, 3);
        assert_eq!(resultado.completadas_como_comprador, 2);
        assert_eq!(resultado.ordenes_como_vendedor, 0);
        assert_eq!(resultado.completadas_como_vendedor, 0);
    }

    #[ink::test]
    fn test_procesar_ordenes_por_usuario_como_vendedor() {
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 3, Estado::Enviado)),
            (3, crear_orden(12, 1, 3, 1, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));

        assert_eq!(resultado.ordenes_como_vendedor, 3);
        assert_eq!(resultado.completadas_como_vendedor, 2);
        assert_eq!(resultado.ordenes_como_comprador, 0);
    }

    #[ink::test]
    fn test_procesar_ordenes_por_usuario_ambos_roles() {
        let ordenes = vec![
            (1, crear_orden(1, 10, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 3, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));

        assert_eq!(resultado.ordenes_como_comprador, 1);
        assert_eq!(resultado.completadas_como_comprador, 1);
        assert_eq!(resultado.ordenes_como_vendedor, 1);
        assert_eq!(resultado.completadas_como_vendedor, 1);
    }

    #[ink::test]
    fn test_procesar_ordenes_por_usuario_sin_ordenes() {
        let ordenes = vec![
            (1, crear_orden(2, 3, 1, 2, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));

        assert_eq!(resultado.ordenes_como_comprador, 0);
        assert_eq!(resultado.ordenes_como_vendedor, 0);
    }

    // ==================== TESTS DE _procesar_resumen_ordenes_todos_usuarios ====================

    #[ink::test]
    fn test_procesar_resumen_ordenes_todos_usuarios() {
        let usuarios = vec![cuenta(1), cuenta(2), cuenta(3)];
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 2, 2, 3, Estado::Pendiente)),
        ];

        let resultado = Reportes::_procesar_resumen_ordenes_todos_usuarios(usuarios, ordenes);

        assert_eq!(resultado.len(), 2);
        
        let u1 = resultado.iter().find(|u| u.usuario == cuenta(1)).unwrap();
        assert_eq!(u1.ordenes_como_comprador, 2);
        assert_eq!(u1.completadas_como_comprador, 1);

        let u2 = resultado.iter().find(|u| u.usuario == cuenta(2)).unwrap();
        assert_eq!(u2.ordenes_como_vendedor, 2);
        assert_eq!(u2.completadas_como_vendedor, 1);
    }

    #[ink::test]
    fn test_procesar_resumen_ordenes_todos_usuarios_sin_ordenes() {
        let usuarios = vec![cuenta(1), cuenta(2)];
        let ordenes: Vec<(u32, Orden)> = Vec::new();

        let resultado = Reportes::_procesar_resumen_ordenes_todos_usuarios(usuarios, ordenes);

        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_resumen_ordenes_excluye_usuarios_sin_ordenes() {
        let usuarios = vec![cuenta(1), cuenta(2), cuenta(3)];
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_resumen_ordenes_todos_usuarios(usuarios, ordenes);

        assert_eq!(resultado.len(), 2);
        assert!(!resultado.iter().any(|u| u.usuario == cuenta(3)));
    }

    // ==================== TESTS DE _procesar_resumen_general ====================

    #[ink::test]
    fn test_procesar_resumen_general_basico() {
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 2, 2, 3, Estado::Pendiente)),
            (3, crear_orden(1, 2, 3, 1, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_resumen_general(5, 10, ordenes);

        assert_eq!(resultado.0, 5);
        assert_eq!(resultado.1, 10);
        assert_eq!(resultado.2, 3);
        assert_eq!(resultado.3, 2);
    }

    #[ink::test]
    fn test_procesar_resumen_general_sin_datos() {
        let ordenes: Vec<(u32, Orden)> = Vec::new();

        let resultado = Reportes::_procesar_resumen_general(0, 0, ordenes);

        assert_eq!(resultado, (0, 0, 0, 0));
    }

    #[ink::test]
    fn test_procesar_resumen_general_sin_completadas() {
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Pendiente)),
            (2, crear_orden(1, 2, 2, 3, Estado::Enviado)),
            (3, crear_orden(1, 2, 3, 1, Estado::Cancelada)),
        ];

        let resultado = Reportes::_procesar_resumen_general(2, 3, ordenes);

        assert_eq!(resultado.2, 3);
        assert_eq!(resultado.3, 0);
    }

    // ==================== TESTS DE _procesar_listar_categorias ====================

    #[ink::test]
    fn test_procesar_listar_categorias_basico() {
        let productos = vec![
            (1, crear_producto(1, "A", "Electrónica", 100)),
            (2, crear_producto(2, "B", "Libros", 50)),
            (3, crear_producto(3, "C", "Electrónica", 200)),
        ];

        let resultado = Reportes::_procesar_listar_categorias(&productos);

        assert_eq!(resultado.len(), 2);
        assert!(resultado.contains(&String::from("Electrónica")));
        assert!(resultado.contains(&String::from("Libros")));
    }

    #[ink::test]
    fn test_procesar_listar_categorias_sin_productos() {
        let productos: Vec<(u32, Producto)> = Vec::new();

        let resultado = Reportes::_procesar_listar_categorias(&productos);

        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_listar_categorias_una_categoria() {
        let productos = vec![
            (1, crear_producto(1, "A", "Única", 100)),
            (2, crear_producto(2, "B", "Única", 200)),
        ];

        let resultado = Reportes::_procesar_listar_categorias(&productos);

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0], "Única");
    }

    #[ink::test]
    fn test_usuario_con_reputacion() {
        let usuario = UsuarioConReputacion {
            usuario: cuenta(1),
            promedio_x100: 450,
            cantidad_calificaciones: 10,
        };

        assert_eq!(usuario.usuario, cuenta(1));
        assert_eq!(usuario.promedio_x100, 450);
        assert_eq!(usuario.cantidad_calificaciones, 10);

        let usuario2 = usuario.clone();
        assert_eq!(usuario, usuario2);
    }

    #[ink::test]
    fn test_producto_vendido() {
        let producto = ProductoVendido {
            id_producto: 1,
            nombre: String::from("Laptop Gaming"),
            categoria: String::from("Electrónica"),
            vendedor: cuenta(5),
            unidades_vendidas: 100,
        };

        assert_eq!(producto.id_producto, 1);
        assert_eq!(producto.nombre, "Laptop Gaming");
        assert_eq!(producto.categoria, "Electrónica");
        assert_eq!(producto.vendedor, cuenta(5));
        assert_eq!(producto.unidades_vendidas, 100);

        let producto2 = producto.clone();
        assert_eq!(producto, producto2);
    }

    #[ink::test]
    fn test_estadisticas_categoria_struct() {
        let stats = EstadisticasCategoria {
            categoria: String::from("Electrónica"),
            total_ventas: 150,
            total_unidades: 500,
            calificacion_promedio_x100: 425,
            cantidad_productos: 25,
        };

        assert_eq!(stats.categoria, "Electrónica");
        assert_eq!(stats.total_ventas, 150);
        assert_eq!(stats.total_unidades, 500);
        assert_eq!(stats.calificacion_promedio_x100, 425);
        assert_eq!(stats.cantidad_productos, 25);

        let stats2 = stats.clone();
        assert_eq!(stats, stats2);
    }

    #[ink::test]
    fn test_ordenes_usuario_struct() {
        let ordenes = OrdenesUsuario {
            usuario: cuenta(10),
            ordenes_como_comprador: 15,
            ordenes_como_vendedor: 25,
            completadas_como_comprador: 12,
            completadas_como_vendedor: 20,
        };

        assert_eq!(ordenes.usuario, cuenta(10));
        assert_eq!(ordenes.ordenes_como_comprador, 15);
        assert_eq!(ordenes.ordenes_como_vendedor, 25);
        assert_eq!(ordenes.completadas_como_comprador, 12);
        assert_eq!(ordenes.completadas_como_vendedor, 20);

        let ordenes2 = ordenes.clone();
        assert_eq!(ordenes, ordenes2);
    }

    #[ink::test]
    fn test_error_categoria_no_encontrada() {
        let error = Error::CategoriaNoEncontrada;
        assert_eq!(error, Error::CategoriaNoEncontrada);
    }

    #[ink::test]
    fn test_error_debug() {
        let error = Error::CategoriaNoEncontrada;
        let _debug_str = format!("{:?}", error);
    }

    // ==================== TESTS DE EDGE CASES ====================

    #[ink::test]
    fn test_procesar_top_vendedores_limite_cero() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((20, 5), (0, 0))),
            (cuenta(2), crear_reputacion((15, 3), (0, 0))),
        ];

        let resultado = Reportes::_procesar_top_vendedores(reputaciones, 0);
        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_top_compradores_limite_cero() {
        let reputaciones = vec![
            (cuenta(1), crear_reputacion((0, 0), (20, 5))),
            (cuenta(2), crear_reputacion((0, 0), (15, 3))),
        ];

        let resultado = Reportes::_procesar_top_compradores(reputaciones, 0);
        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_limite_cero() {
        let ordenes = vec![(1u32, crear_orden(1, 2, 1, 5, Estado::Recibido))];
        let productos = vec![(1u32, crear_producto(2, "Prod", "Cat", 100))];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 0);
        assert!(resultado.is_empty());
    }

    #[ink::test]
    fn test_procesar_estadisticas_categoria_ordenes_canceladas_no_cuentan() {
        let productos = vec![(1u32, crear_producto(1, "Prod", "Cat", 100))];
        let ordenes = vec![
            (1u32, crear_orden(2, 1, 1, 5, Estado::Cancelada)),
            (2u32, crear_orden(3, 1, 1, 3, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos,
            ordenes,
            String::from("Cat"),
            (20, 5),
        );

        assert!(resultado.is_ok());
        let stats = resultado.unwrap();
        // Solo la orden Recibido cuenta
        assert_eq!(stats.total_ventas, 1);
        assert_eq!(stats.total_unidades, 3);
    }

    #[ink::test]
    fn test_procesar_resumen_general_ordenes_canceladas_no_cuentan() {
        let ordenes = vec![
            (1u32, crear_orden(1, 2, 1, 5, Estado::Cancelada)),
            (2u32, crear_orden(1, 2, 1, 3, Estado::Recibido)),
            (3u32, crear_orden(1, 2, 1, 2, Estado::Pendiente)),
        ];

        let resultado = Reportes::_procesar_resumen_general(5, 10, ordenes);

        assert_eq!(resultado.0, 5);  // total usuarios
        assert_eq!(resultado.1, 10); // total productos
        assert_eq!(resultado.2, 3);  // total órdenes
        assert_eq!(resultado.3, 1);  // solo 1 completada (Recibido)
    }
}
