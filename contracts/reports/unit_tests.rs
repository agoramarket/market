mod tests {
        use super::*;

        /// Crea una cuenta de prueba con un byte específico.
        fn cuenta(n: u8) -> AccountId {
            AccountId::from([n; 32])
        }

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

        #[ink::test]
        fn test_ordenar_por_reputacion_basico() {
            let reportes = Reportes::new(cuenta(1));

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

            reportes._ordenar_por_reputacion(&mut usuarios);

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
            let reportes = Reportes::new(cuenta(1));
            let mut usuarios: Vec<UsuarioConReputacion> = Vec::new();

            reportes._ordenar_por_reputacion(&mut usuarios);

            assert!(usuarios.is_empty());
        }

        #[ink::test]
        fn test_ordenar_por_reputacion_un_elemento() {
            let reportes = Reportes::new(cuenta(1));
            let mut usuarios = vec![UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 400,
                cantidad_calificaciones: 5,
            }];

            reportes._ordenar_por_reputacion(&mut usuarios);

            assert_eq!(usuarios.len(), 1);
            assert_eq!(usuarios[0].promedio_x100, 400);
        }

        #[ink::test]
        fn test_ordenar_por_reputacion_empate_total() {
            let reportes = Reportes::new(cuenta(1));
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

            reportes._ordenar_por_reputacion(&mut usuarios);

            assert_eq!(usuarios.len(), 2);
            assert_eq!(usuarios[0].promedio_x100, 450);
            assert_eq!(usuarios[1].promedio_x100, 450);
        }

        #[ink::test]
        fn test_ordenar_desempate_por_cantidad_calificaciones() {
            let reportes = Reportes::new(cuenta(1));
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

            reportes._ordenar_por_reputacion(&mut usuarios);

            assert_eq!(usuarios[0].cantidad_calificaciones, 20);
            assert_eq!(usuarios[1].cantidad_calificaciones, 10);
            assert_eq!(usuarios[2].cantidad_calificaciones, 5);
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
        fn test_usuario_con_reputacion_valores_extremos() {
            let min = UsuarioConReputacion {
                usuario: cuenta(0),
                promedio_x100: 100,
                cantidad_calificaciones: 1,
            };
            assert_eq!(min.promedio_x100, 100);

            let max = UsuarioConReputacion {
                usuario: cuenta(255),
                promedio_x100: 500,
                cantidad_calificaciones: u32::MAX,
            };
            assert_eq!(max.promedio_x100, 500);
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
        fn test_producto_vendido_nombre_largo() {
            let nombre_largo = "A".repeat(256);
            let producto = ProductoVendido {
                id_producto: 999,
                nombre: nombre_largo.clone(),
                categoria: String::from("Test"),
                vendedor: cuenta(1),
                unidades_vendidas: 1,
            };

            assert_eq!(producto.nombre.len(), 256);
        }

        #[ink::test]
        fn test_estadisticas_categoria() {
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
        fn test_estadisticas_categoria_sin_ventas() {
            let stats = EstadisticasCategoria {
                categoria: String::from("Nueva Categoría"),
                total_ventas: 0,
                total_unidades: 0,
                calificacion_promedio_x100: 0,
                cantidad_productos: 5,
            };

            assert_eq!(stats.total_ventas, 0);
            assert_eq!(stats.calificacion_promedio_x100, 0);
        }

        #[ink::test]
        fn test_ordenes_usuario() {
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
        fn test_ordenes_usuario_sin_actividad() {
            let ordenes = OrdenesUsuario {
                usuario: cuenta(1),
                ordenes_como_comprador: 0,
                ordenes_como_vendedor: 0,
                completadas_como_comprador: 0,
                completadas_como_vendedor: 0,
            };

            assert_eq!(ordenes.ordenes_como_comprador, 0);
            assert_eq!(ordenes.ordenes_como_vendedor, 0);
        }

        #[ink::test]
        fn test_ordenes_usuario_solo_comprador() {
            let ordenes = OrdenesUsuario {
                usuario: cuenta(1),
                ordenes_como_comprador: 10,
                ordenes_como_vendedor: 0,
                completadas_como_comprador: 8,
                completadas_como_vendedor: 0,
            };

            assert!(ordenes.ordenes_como_comprador > 0);
            assert_eq!(ordenes.ordenes_como_vendedor, 0);
        }

        #[ink::test]
        fn test_ordenes_usuario_solo_vendedor() {
            let ordenes = OrdenesUsuario {
                usuario: cuenta(1),
                ordenes_como_comprador: 0,
                ordenes_como_vendedor: 20,
                completadas_como_comprador: 0,
                completadas_como_vendedor: 18,
            };

            assert_eq!(ordenes.ordenes_como_comprador, 0);
            assert!(ordenes.ordenes_como_vendedor > 0);
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

        #[ink::test]
        fn test_calculo_promedio_x100() {
            let suma = 20u32;
            let cantidad = 5u32;
            let promedio_x100 = (suma * 100) / cantidad;
            assert_eq!(promedio_x100, 400);

            let suma2 = 12u32;
            let cantidad2 = 3u32;
            let promedio2 = (suma2 * 100) / cantidad2;
            assert_eq!(promedio2, 400);
        }

        #[ink::test]
        fn test_calculo_promedio_con_decimales() {
            let suma = 7u32;
            let cantidad = 3u32;
            let promedio_x100 = (suma * 100) / cantidad;
            assert_eq!(promedio_x100, 233);
        }

        #[ink::test]
        fn test_comparacion_usuarios() {
            let u1 = UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 400,
                cantidad_calificaciones: 10,
            };

            let u2 = UsuarioConReputacion {
                usuario: cuenta(1),
                promedio_x100: 400,
                cantidad_calificaciones: 10,
            };

            let u3 = UsuarioConReputacion {
                usuario: cuenta(2),
                promedio_x100: 400,
                cantidad_calificaciones: 10,
            };

            assert_eq!(u1, u2);
            assert_ne!(u1, u3);
        }

        #[ink::test]
        fn test_comparacion_productos() {
            let p1 = ProductoVendido {
                id_producto: 1,
                nombre: String::from("Test"),
                categoria: String::from("Cat"),
                vendedor: cuenta(1),
                unidades_vendidas: 10,
            };

            let p2 = p1.clone();
            assert_eq!(p1, p2);

            let p3 = ProductoVendido {
                id_producto: 2,
                nombre: String::from("Test"),
                categoria: String::from("Cat"),
                vendedor: cuenta(1),
                unidades_vendidas: 10,
            };
            assert_ne!(p1, p3);
        }

        #[ink::test]
        fn test_comparacion_estadisticas() {
            let s1 = EstadisticasCategoria {
                categoria: String::from("Cat1"),
                total_ventas: 100,
                total_unidades: 200,
                calificacion_promedio_x100: 450,
                cantidad_productos: 10,
            };

            let s2 = s1.clone();
            assert_eq!(s1, s2);

            let s3 = EstadisticasCategoria {
                categoria: String::from("Cat2"),
                total_ventas: 100,
                total_unidades: 200,
                calificacion_promedio_x100: 450,
                cantidad_productos: 10,
            };
            assert_ne!(s1, s3);
        }

        #[ink::test]
        fn test_comparacion_ordenes() {
            let o1 = OrdenesUsuario {
                usuario: cuenta(1),
                ordenes_como_comprador: 5,
                ordenes_como_vendedor: 10,
                completadas_como_comprador: 4,
                completadas_como_vendedor: 8,
            };

            let o2 = o1.clone();
            assert_eq!(o1, o2);

            let o3 = OrdenesUsuario {
                usuario: cuenta(2),
                ordenes_como_comprador: 5,
                ordenes_como_vendedor: 10,
                completadas_como_comprador: 4,
                completadas_como_vendedor: 8,
            };
            assert_ne!(o1, o3);
        }

        #[ink::test]
        fn test_ordenar_muchos_usuarios() {
            let reportes = Reportes::new(cuenta(1));
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

            reportes._ordenar_por_reputacion(&mut usuarios);

            assert_eq!(usuarios[0].promedio_x100, 500);
            assert_eq!(usuarios[1].promedio_x100, 450);
            assert_eq!(usuarios[2].promedio_x100, 350);
            assert_eq!(usuarios[3].promedio_x100, 200);
            assert_eq!(usuarios[4].promedio_x100, 100);
        }

        #[ink::test]
        fn test_ordenar_preserva_todos_elementos() {
            let reportes = Reportes::new(cuenta(1));
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
            reportes._ordenar_por_reputacion(&mut usuarios);

            assert_eq!(usuarios.len(), len_original);
        }
}