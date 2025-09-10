/*
|--------------------------------------------------------------------------
| Routes file
|--------------------------------------------------------------------------
|
| The routes file is used for defining the HTTP routes.
|
*/

import router from '@adonisjs/core/services/router'

router.get('/', async () => {
  return {
    hello: 'world',
  }
})

router
  .group(() => {
    router.post('/shorten', '#controllers/links_controller.shorten')
    router.get('/redirect/:shortUrl', '#controllers/links_controller.redirect')
  })
  .prefix('/api')
