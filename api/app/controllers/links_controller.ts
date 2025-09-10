import type { HttpContext } from '@adonisjs/core/http'
import Link from '#models/link'
import { randomBytes } from 'node:crypto'

export default class LinksController {
  private createShortUrl(): string {
    return randomBytes(3).toString('hex')
  }

  async shorten({ request, response }: HttpContext) {
    const { url } = request.only(['url'])

    if (!url) {
      return response.badRequest({ error: 'URL is required' })
    }

    if (!url.startsWith('gurt://')) {
      return response.badRequest({ error: 'Only gurt:// URLs are supported' })
    }

    const existingLink = await Link.findBy('url', url)
    if (existingLink) {
      return response.ok({
        user: {
          short_url: existingLink.shortUrl,
        },
      })
    }

    const shortUrl = this.createShortUrl()
    const link = await Link.create({
      url,
      shortUrl,
    })

    return response.created({
      user: {
        short_url: link.shortUrl,
      },
    })
  }

  async redirect({ params, response }: HttpContext) {
    const { shortUrl } = params

    const link = await Link.findBy('shortUrl', shortUrl)
    if (!link) {
      return response.notFound({ error: 'Short URL not found' })
    }

    link.clickCount += 1
    await link.save()

    return response.ok({
      url: link.url,
    })
  }
}
